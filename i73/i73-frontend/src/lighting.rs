use bit_vec::BitVec;

use i73_base::Block;

use lumis::heightmap::{ChunkHeightMap, ColumnHeightMap, HeightMapBuilder};
use lumis::light::Lighting;
use lumis::sources::SkyLightSources;
use lumis::queue::Queue;

use rayon::iter::ParallelBridge;
use rayon::prelude::{ParallelIterator, IntoParallelRefIterator};

use std::collections::HashMap;
use std::mem;
use std::sync::Mutex;
use std::time::Instant;

use vocs::component::LayerStorage;
use vocs::indexed::ChunkIndexed;
use vocs::mask::ChunkMask;
use vocs::mask::LayerMask;
use vocs::nibbles::{u4, BulkNibbles, ChunkNibbles};
use vocs::position::{dir, Offset, ChunkPosition, GlobalColumnPosition, GlobalSectorPosition, LayerPosition};
use vocs::view::{Directional, SplitDirectional};
use vocs::world::sector::Sector;
use vocs::world::shared::{NoPack, SharedSector, SharedWorld};
use vocs::world::world::World;

struct Layer<T>(Box<[T]>);

impl<T> Default for Layer<T> where T: Default {
	fn default() -> Self {
		let values: Vec<T> = (0..256).map(|_| T::default()).collect();

		Layer(values.into_boxed_slice())
	}
}

impl<T> Layer<T> {
	fn map<F, V>(self, mapper: F) -> Layer<V> where F: FnMut(T) -> V {
		let entries: Vec<V> = self.0.into_vec().into_iter().map(mapper).collect();

		Layer(entries.into_boxed_slice())
	}

	fn into_inner(self) -> Box<[T]> {
		self.0
	}
}

impl<T> Layer<T> where T: Clone {
	fn filled(value: T) -> Self {
		Layer(vec![value; 256].into_boxed_slice())
	}
}

impl<T> std::ops::Index<LayerPosition> for Layer<T> {
	type Output = T;

	fn index(&self, index: LayerPosition) -> &Self::Output {
		&self.0[index.zx() as usize]
	}
}

impl<T> std::ops::IndexMut<LayerPosition> for Layer<T> {
	fn index_mut(&mut self, index: LayerPosition) -> &mut Self::Output {
		&mut self.0[index.zx() as usize]
	}
}

struct SectorQueue {
	/// The queue currently being emptied.
	front: Sector<ChunkMask>,
	/// The queue currently being filled.
	back: Sector<ChunkMask>,
	spills: Directional<Layer<Option<LayerMask>>>
}

impl SectorQueue {
	fn new() -> Self {
		SectorQueue {
			front: Sector::new(),
			back: Sector::new(),
			spills: Directional::combine(SplitDirectional {
				plus_x: Layer::default(),
				minus_x: Layer::default(),
				up: Layer::default(),
				down: Layer::default(),
				plus_z: Layer::default(),
				minus_z: Layer::default()
			})
		}
	}

	fn spill<D, F>(&mut self, origin: ChunkPosition, dir: D, layer: LayerMask, mut f: F)
		where ChunkPosition: Offset<D, Spill = LayerPosition>,
			F: FnMut(&mut ChunkMask, LayerMask),
			D: Copy,
			Directional<Layer<Option<LayerMask>>>: std::ops::IndexMut<D, Output = Layer<Option<LayerMask>>> {


		// If the layer is empty, don't bother adding / merging it.
		if layer.is_filled(false) {
			return;
		}

		// Either merge it with a local chunk mask, or add it to the neighboring spills.
		match origin.offset_spilling(dir) {
			Ok(position) => f(self.back.get_or_create_mut(position), layer),
			Err(spilled) => {
				let slot = &mut self.spills[dir][spilled];

				match slot.as_mut() {
					Some(existing) => *existing |= &layer,
					None => *slot = Some(layer)
				}
			}
		}
	}

	fn spill_out(&mut self, origin: ChunkPosition, spills: SplitDirectional<LayerMask>) {
		self.spill(origin, dir::Up, spills.up, |mask, layer| mask.layer_zx_mut(0).combine(&layer));
		self.spill(origin, dir::Down, spills.down, |mask, layer| mask.layer_zx_mut(15).combine(&layer));
		self.spill(origin, dir::PlusX, spills.plus_x, |mask, layer| mask.layer_zy_mut(0).combine(&layer));
		self.spill(origin, dir::MinusX, spills.minus_x, |mask, layer| mask.layer_zy_mut(15).combine(&layer));
		self.spill(origin, dir::PlusZ, spills.plus_z, |mask, layer| mask.layer_yx_mut(0).combine(&layer));
		self.spill(origin, dir::MinusZ, spills.minus_z, |mask, layer| mask.layer_yx_mut(15).combine(&layer));
	}

	fn reset_spills(&mut self) -> SplitDirectional<Layer<Option<LayerMask>>> {
		assert!(self.front.is_empty());

		std::mem::replace(&mut self.spills, Directional::combine(SplitDirectional {
			plus_x: Layer::default(),
			minus_x: Layer::default(),
			up: Layer::default(),
			down: Layer::default(),
			plus_z: Layer::default(),
			minus_z: Layer::default()
		})).split()
	}

	fn empty(&self) -> bool {
		self.front.is_empty()
	}

	fn pop(&mut self) -> Option<(ChunkPosition, ChunkMask)> {
		self.front.pop_first()
	}

	fn flip(&mut self) -> bool {
		mem::swap(&mut self.front, &mut self.back);

		!self.front.is_empty()
	}
}

fn lighting_info() -> HashMap<Block, u4> {
	let mut lighting_info = HashMap::new();

	lighting_info.insert(Block::air(), u4::new(0));
	lighting_info.insert(Block::from_anvil_id(8 * 16), u4::new(2));
	lighting_info.insert(Block::from_anvil_id(9 * 16), u4::new(2));
	lighting_info.insert(Block::from_anvil_id(18 * 16), u4::new(1));

	lighting_info
}

fn initial_sector(block_sector: &Sector<ChunkIndexed<Block>>, sky_light: &SharedSector<NoPack<ChunkNibbles>>) -> (SectorQueue, Layer<ColumnHeightMap>) {
	let lighting_info = lighting_info();
	let empty_lighting = ChunkNibbles::default();
	let sector_queue = Mutex::new(SectorQueue::new());

	let unordered_heightmaps: Vec<(LayerPosition, ColumnHeightMap)> = 
		block_sector.enumerate_columns().par_bridge().map(|(position, column)| {

		let mut mask = LayerMask::default();
		let mut heightmap_builder = HeightMapBuilder::new();
		// TODO: Reuse this!
		let mut queue = Queue::default();

		for (y, chunk) in column.iter().enumerate().rev() {
			let (blocks, palette) = match chunk {
				&Some(chunk) => chunk.freeze(),
				&None => unimplemented!()
			};

			let mut opacity = BulkNibbles::new(palette.len());
			let mut obstructs = BitVec::new();

			for (index, value) in palette.iter().enumerate() {
				let opacity_value = value
				.and_then(|entry| lighting_info.get(&entry).map(|opacity| *opacity))
				.unwrap_or(u4::new(15));
				
				opacity.set(index, opacity_value);

				obstructs.push(opacity_value != u4::new(0));
			}

			let chunk_heightmap = ChunkHeightMap::build(blocks, &obstructs, mask);
			let sources = SkyLightSources::new(&chunk_heightmap);

			let mut light_data = ChunkNibbles::default();
			let neighbors = Directional::combine(SplitDirectional {
				minus_x: &empty_lighting,
				plus_x: &empty_lighting,
				minus_z: &empty_lighting,
				plus_z: &empty_lighting,
				down: &empty_lighting,
				up: &empty_lighting,
			});

			let mut light = Lighting::new(&mut light_data, neighbors, sources, opacity);

			light.initial(&mut queue);
			light.apply(blocks, &mut queue);

			mask = heightmap_builder.add(chunk_heightmap);

			let old_spills = queue.reset_spills().split();

			let chunk_position = ChunkPosition::from_layer(y as u8, position);

			sector_queue.lock().unwrap().spill_out(chunk_position, old_spills);
			sky_light.set(chunk_position, NoPack(light_data));
		}

		(position, heightmap_builder.build())
	}).collect();

	// We've received an unordered list of heightmaps from the parallel iterator.
	// It's necessary to properly sort them before returning.
	// First, we order them with the ordered_heightmaps layer...
	let mut ordered_heightmaps: Layer<Option<ColumnHeightMap>> = Layer::default();

	for (position, heightmap) in unordered_heightmaps {
		ordered_heightmaps[position] = Some(heightmap);
	}

	// ... then, we unwrap all of the heightmaps, since at this point every slot should
	// be occupied by a Some value.
	let heightmaps = ordered_heightmaps.map(Option::unwrap);

	(sector_queue.into_inner().unwrap(), heightmaps)
}

fn full_sector(
	block_sector: &Sector<ChunkIndexed<Block>>, 
	sky_light: &SharedSector<NoPack<ChunkNibbles>>, 
	sky_light_neighbors: Directional<&SharedSector<NoPack<ChunkNibbles>>>, 
	sector_queue: &mut SectorQueue, 
	heightmaps: &Layer<ColumnHeightMap>) -> (u32, u32) {

	let mut iterations = 0;
	let mut chunk_operations = 0;

	while sector_queue.flip() {
		iterations += 1;

		while let Some((position, incomplete)) = sector_queue.pop() {
			chunk_operations += 1;

			let blocks = block_sector[position].as_ref().unwrap();
			let column_heightmap = &heightmaps[position.layer()];

			let heightmap = column_heightmap.slice(u4::new(position.y()));

			let mut queue = complete_chunk(position, blocks, sky_light, sky_light_neighbors, incomplete, &heightmap);

			sector_queue.spill_out(position, queue.reset_spills().split());
		}
	}

	(iterations, chunk_operations)
}

fn complete_chunk (
	position: ChunkPosition, 
	blocks: &ChunkIndexed<Block>, 
	sky_light: &SharedSector<NoPack<ChunkNibbles>>, 
	sky_light_neighbors: Directional<&SharedSector<NoPack<ChunkNibbles>>>, 
	incomplete: ChunkMask, 
	heightmap: &ChunkHeightMap) -> Queue {

	// TODO: Cache these things!
	let lighting_info = lighting_info();
	let empty_lighting = ChunkNibbles::default();
	let mut queue = Queue::default();

	let (blocks, palette) = blocks.freeze();

	let mut opacity = BulkNibbles::new(palette.len());

	for (index, value) in palette.iter().enumerate() {
		opacity.set(
			index,
			value
				.and_then(|entry| lighting_info.get(&entry).map(|opacity| *opacity))
				.unwrap_or(u4::new(15)),
		);
	}

	let sources = SkyLightSources::new(heightmap);

	let mut central = sky_light.get_or_create(position);
	let locks = SplitDirectional {
		up: position
			.offset(dir::Up)
			.map(|position| sky_light[position].read())
			.unwrap_or_else(|| sky_light_neighbors[dir::Up][position.offset_wrapping(dir::Up)].read()),
		down: position
			.offset(dir::Down)
			.map(|position| sky_light[position].read())
			.unwrap_or_else(|| sky_light_neighbors[dir::Down][position.offset_wrapping(dir::Down)].read()),
		plus_x: position
			.offset(dir::PlusX)
			.map(|position| sky_light[position].read())
			.unwrap_or_else(|| sky_light_neighbors[dir::PlusX][position.offset_wrapping(dir::PlusX)].read()),
		minus_x: position
			.offset(dir::MinusX)
			.map(|position| sky_light[position].read())
			.unwrap_or_else(|| sky_light_neighbors[dir::MinusX][position.offset_wrapping(dir::MinusX)].read()),
		plus_z: position
			.offset(dir::PlusZ)
			.map(|position| sky_light[position].read())
			.unwrap_or_else(|| sky_light_neighbors[dir::PlusZ][position.offset_wrapping(dir::PlusZ)].read()),
		minus_z: position
			.offset(dir::MinusZ)
			.map(|position| sky_light[position].read())
			.unwrap_or_else(|| sky_light_neighbors[dir::MinusZ][position.offset_wrapping(dir::MinusZ)].read())
	};

	let neighbors = SplitDirectional {
		up: locks
			.up
			.as_ref()
			.map(|chunk| &chunk.0)
			.unwrap_or(&empty_lighting),
		down: locks
			.down
			.as_ref()
			.map(|chunk| &chunk.0)
			.unwrap_or(&empty_lighting),
		plus_x: locks
			.plus_x
			.as_ref()
			.map(|chunk| &chunk.0)
			.unwrap_or(&empty_lighting),
		minus_x: locks
			.minus_x
			.as_ref()
			.map(|chunk| &chunk.0)
			.unwrap_or(&empty_lighting),
		plus_z: locks
			.plus_z
			.as_ref()
			.map(|chunk| &chunk.0)
			.unwrap_or(&empty_lighting),
		minus_z: locks
			.minus_z
			.as_ref()
			.map(|chunk| &chunk.0)
			.unwrap_or(&empty_lighting),
	};

	let mut light =
		Lighting::new(&mut central, Directional::combine(neighbors), sources, opacity);

	queue.reset_from_mask(incomplete);
	light.apply(blocks, &mut queue);

	queue
}

pub fn compute_skylight(world: &World<ChunkIndexed<Block>>) -> (SharedWorld<NoPack<ChunkNibbles>>, HashMap<GlobalColumnPosition, ColumnHeightMap>) {
	let empty_sector: SharedSector<NoPack<ChunkNibbles>> = SharedSector::new();

	let mut sky_light: SharedWorld<NoPack<ChunkNibbles>> = SharedWorld::new();
	let heightmaps: Mutex<HashMap<GlobalSectorPosition, Layer<ColumnHeightMap>>> = Mutex::new(HashMap::new());
	let sector_queues: Mutex<HashMap<GlobalSectorPosition, SectorQueue>> = Mutex::new(HashMap::new());

	let positions = [
		GlobalSectorPosition::new(0, 0),
		GlobalSectorPosition::new(0, 1),
		GlobalSectorPosition::new(1, 0),
		GlobalSectorPosition::new(1, 1)
	];

	for &position in &positions {
		sky_light.get_or_create_sector_mut(position);
	}

	positions.par_iter().for_each(|position| {
		let position = *position;

		println!("Performing initial lighting for sector ({}, {})", position.x(), position.z());
		let initial_start = Instant::now();

		let block_sector = match world.get_sector(position) {
			Some(sector) => sector,
			None => return
		};

		let sky_light = sky_light.get_sector(position).unwrap();

		let (mut sector_queue, sector_heightmaps) = initial_sector(block_sector, sky_light);

		{
			let end = ::std::time::Instant::now();
			let time = end.duration_since(initial_start);
	
			let secs = time.as_secs();
			let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);
	
			println!("Initial sky lighting for ({}, {}) done in {}us ({}us per column)", position.x(), position.z(), us, us / 256);
		}

		println!("Performing inner full sky lighting for sector ({}, {})", position.x(), position.z());
		let inner_start = Instant::now();

		let sky_light_neighbors = Directional::combine(SplitDirectional {
			minus_x: &empty_sector,
			plus_x: &empty_sector,
			minus_z: &empty_sector,
			plus_z: &empty_sector,
			down: &empty_sector,
			up: &empty_sector
		});

		let (iterations, chunk_operations) = full_sector(block_sector, sky_light, sky_light_neighbors, &mut sector_queue, &sector_heightmaps);

		sector_queues.lock().unwrap().insert(position, sector_queue);
		heightmaps.lock().unwrap().insert(position, sector_heightmaps);

		{
			let end = ::std::time::Instant::now();
			let time = end.duration_since(inner_start);
	
			let secs = time.as_secs();
			let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);
	
			println!("Inner full sky lighting done in {}us ({}us per column): {} iterations, {} post-initial chunk light operations", us, us / 256, iterations, chunk_operations);
		}
	});

	let mut heightmaps = heightmaps.into_inner().unwrap();

	let complete_sector = |position: GlobalSectorPosition| {
		println!("Performing full sky lighting for sector ({}, {})", position.x(), position.z());

		let full_start = Instant::now();

		let block_sector = match world.get_sector(position) {
			Some(sector) => sector,
			None => return
		};

		let sky_light_center = sky_light.get_sector(position).unwrap();

		let sky_light_neighbors = Directional::combine(SplitDirectional {
			minus_x: sky_light.get_sector(GlobalSectorPosition::new(position.x() - 1, position.z())).unwrap_or(&empty_sector),
			plus_x: sky_light.get_sector(GlobalSectorPosition::new(position.x() + 1, position.z())).unwrap_or(&empty_sector),
			minus_z: sky_light.get_sector(GlobalSectorPosition::new(position.x(), position.z() - 1)).unwrap_or(&empty_sector),
			plus_z: sky_light.get_sector(GlobalSectorPosition::new(position.x(), position.z() + 1)).unwrap_or(&empty_sector),
			down: &empty_sector,
			up: &empty_sector,
		});

		let mut sector_queue = sector_queues.lock().unwrap().remove(&position).unwrap();
		let sector_heightmaps = heightmaps.get(&position).unwrap();

		let (iterations, chunk_operations) = full_sector(block_sector, sky_light_center, sky_light_neighbors, &mut sector_queue, sector_heightmaps);

		// TODO: Don't throw away the spills from the sector queue

		{
			let end = ::std::time::Instant::now();
			let time = end.duration_since(full_start);
	
			let secs = time.as_secs();
			let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);
	
			println!("Full sky lighting done in {}us ({}us per column): {} iterations, {} post-initial chunk light operations", us, us / 256, iterations, chunk_operations);
		}
	};

	rayon::join(|| complete_sector(positions[0]), || complete_sector(positions[3]));
	rayon::join(|| complete_sector(positions[1]), || complete_sector(positions[2]));

	// Split up the heightmaps into the format expected by the rest of i73
	let mut individual_heightmaps: HashMap<GlobalColumnPosition, ColumnHeightMap> = HashMap::new();

	for &position in &positions {
		let sector_heightmaps = heightmaps.remove(&position).unwrap();

		for (index, heightmap) in sector_heightmaps.into_inner().into_vec().into_iter().enumerate() {
			let layer = LayerPosition::from_zx(index as u8);
			let column_position = GlobalColumnPosition::combine(position, layer);
	
			individual_heightmaps.insert(column_position, heightmap);
		}
	}

	(sky_light, individual_heightmaps)
}
