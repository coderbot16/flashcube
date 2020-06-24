use bit_vec::BitVec;

use i73_base::Block;

use lumis::heightmap::{ChunkHeightMap, ColumnHeightMap, HeightMapBuilder};
use lumis::light::Lighting;
use lumis::sources::SkyLightSources;
use lumis::queue::Queue;

use rayon::iter::ParallelBridge;
use rayon::prelude::ParallelIterator;

use std::collections::HashMap;
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

struct SectorSpills {
	local: Sector<ChunkMask>,
	spills: Directional<Layer<Option<LayerMask>>>
}

impl SectorSpills {
	fn new() -> Self {
		SectorSpills {
			local: Sector::new(),
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

		if layer.is_filled(false) {
			return;
		}

		match origin.offset_spilling(dir) {
			Ok(position) => f(self.local.get_or_create_mut(position), layer),
			Err(spilled) => {
				if self.spills[dir][spilled].is_none() {
					todo!("Cannot merge spilled LayerMasks yet");
				}

				self.spills[dir][spilled] = Some(layer);
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

	fn reset(&mut self) -> SplitDirectional<Layer<Option<LayerMask>>> {
		assert!(self.local.is_empty());

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
		self.local.is_empty()
	}

	fn pop(&mut self) -> Option<(ChunkPosition, ChunkMask)> {
		/*let position = match self.queued.primary.pop_first() {
			Some(position) => position,
			None => return None
		};

		let mut mask = ChunkMask::default();

		let incoming = SplitDirectional {
			plus_x:  position.offset(dir::PlusX ).and_then(|offset| self.layers.minus_x.get(offset)),
			minus_x: position.offset(dir::MinusX).and_then(|offset| self.layers.plus_x.get(offset)),
			down:    position.offset(dir::Down  ).and_then(|offset| self.layers.up.get(offset)),
			up:      position.offset(dir::Up    ).and_then(|offset| self.layers.down.get(offset)),
			plus_z:  position.offset(dir::PlusZ ).and_then(|offset| self.layers.minus_z.get(offset)),
			minus_z: position.offset(dir::MinusZ).and_then(|offset| self.layers.plus_z.get(offset))
		};
		
		incoming.plus_x.map(|layer| mask.layer_zy_mut(15).combine(&layer));
		incoming.minus_x.map(|layer| mask.layer_zy_mut(0).combine(&layer));
		incoming.up.map(|layer| mask.layer_zx_mut(15).combine(&layer));
		incoming.down.map(|layer| mask.layer_zx_mut(0).combine(&layer));
		incoming.plus_z.map(|layer| mask.layer_yx_mut(15).combine(&layer));
		incoming.minus_z.map(|layer| mask.layer_yx_mut(0).combine(&layer));*/

		self.local.pop_first()
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

fn initial_sector(block_sector: &Sector<ChunkIndexed<Block>>, sky_light: &SharedSector<NoPack<ChunkNibbles>>) -> (SectorSpills, Layer<ColumnHeightMap>) {
	let lighting_info = lighting_info();
	let empty_lighting = ChunkNibbles::default();
	let spills = Mutex::new(SectorSpills::new());

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

			spills.lock().unwrap().spill_out(chunk_position, old_spills);
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

	(spills.into_inner().unwrap(), heightmaps)
}

fn full_sector(block_sector: &Sector<ChunkIndexed<Block>>, sky_light: &SharedSector<NoPack<ChunkNibbles>>) -> Layer<ColumnHeightMap> {
	let initial_start = Instant::now();

	let (mut spills, heightmaps) = initial_sector(block_sector, sky_light);
	
	let mut new_spills = SectorSpills::new();

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(initial_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Initial sky lighting done in {}us ({}us per column)", us, us / 256);
	}

	let full_start = Instant::now();

	let mut iterations = 0;
	let mut chunk_operations = 0;

	while !spills.empty() {
		iterations += 1;

		while let Some((position, incomplete)) = spills.pop() {
			chunk_operations += 1;

			let blocks = block_sector[position].as_ref().unwrap();
			let column_heightmap = &heightmaps[position.layer()];

			let heightmap = column_heightmap.slice(u4::new(position.y()));

			let mut queue = complete_chunk(position, blocks, sky_light, incomplete, &heightmap);

			new_spills.spill_out(position, queue.reset_spills().split());
		}

		spills.reset();
		std::mem::swap(&mut spills, &mut new_spills);
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(full_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Full sky lighting done in {}us ({}us per column): {} iterations, {} post-initial chunk light operations", us, us / 256, iterations, chunk_operations);
	}

	heightmaps
}

fn complete_chunk(position: ChunkPosition, blocks: &ChunkIndexed<Block>, sky_light: &SharedSector<NoPack<ChunkNibbles>>, incomplete: ChunkMask, heightmap: &ChunkHeightMap) -> Queue {
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

	// TODO: cross-sector lighting

	let mut central = sky_light.get_or_create(position);
	let locks = SplitDirectional {
		up: position.offset(dir::Up).map(|position| sky_light[position].read()),
		down: position.offset(dir::Down).map(|position| sky_light[position].read()),
		plus_x: position
			.offset(dir::PlusX)
			.map(|position| sky_light[position].read()),
		minus_x: position
			.offset(dir::MinusX)
			.map(|position| sky_light[position].read()),
		plus_z: position
			.offset(dir::PlusZ)
			.map(|position| sky_light[position].read()),
		minus_z: position
			.offset(dir::MinusZ)
			.map(|position| sky_light[position].read()),
	};

	let neighbors = SplitDirectional {
		up: locks
			.up
			.as_ref()
			.and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0))
			.unwrap_or(&empty_lighting),
		down: locks
			.down
			.as_ref()
			.and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0))
			.unwrap_or(&empty_lighting),
		plus_x: locks
			.plus_x
			.as_ref()
			.and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0))
			.unwrap_or(&empty_lighting),
		minus_x: locks
			.minus_x
			.as_ref()
			.and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0))
			.unwrap_or(&empty_lighting),
		plus_z: locks
			.plus_z
			.as_ref()
			.and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0))
			.unwrap_or(&empty_lighting),
		minus_z: locks
			.minus_z
			.as_ref()
			.and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0))
			.unwrap_or(&empty_lighting),
	};

	let mut light =
		Lighting::new(&mut central, Directional::combine(neighbors), sources, opacity);

	queue.reset_from_mask(incomplete);
	light.apply(blocks, &mut queue);

	queue
}

pub fn compute_skylight(world: &World<ChunkIndexed<Block>>) -> (SharedWorld<NoPack<ChunkNibbles>>, HashMap<GlobalColumnPosition, ColumnHeightMap>) {
	let mut sky_light: SharedWorld<NoPack<ChunkNibbles>> = SharedWorld::new();
	let mut heightmaps: HashMap<GlobalColumnPosition, ColumnHeightMap> = HashMap::new(); // TODO: Better vocs integration.

	for sector_z in 0..2 {
		for sector_x in 0..2 {
			println!("Performing sky lighting for sector ({}, {})", sector_x, sector_z);

			let position = GlobalSectorPosition::new(sector_x, sector_z);

			let block_sector = match world.get_sector(position) {
				Some(sector) => sector,
				None => continue
			};

			let sky_light = sky_light.get_or_create_sector_mut(position);

			let sector_heightmaps = full_sector(block_sector, sky_light);

			for (index, heightmap) in sector_heightmaps.into_inner().into_vec().into_iter().enumerate() {
				let layer = LayerPosition::from_zx(index as u8);
				let column_position = GlobalColumnPosition::combine(position, layer);

				heightmaps.insert(column_position, heightmap);
			}
		}
	}

	(sky_light, heightmaps)
}
