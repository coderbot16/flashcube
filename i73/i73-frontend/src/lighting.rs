use bit_vec::BitVec;

use i73_base::Block;

use lumis::heightmap::{ChunkHeightMap, ColumnHeightMap, HeightMapBuilder};
use lumis::light::Lighting;
use lumis::sources::SkyLightSources;
use lumis::queue::{ChunkQueue, SectorQueue, SectorSpills};

use rayon::iter::ParallelBridge;
use rayon::prelude::{ParallelIterator, IntoParallelRefIterator};

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use vocs::indexed::ChunkIndexed;
use vocs::mask::ChunkMask;
use vocs::mask::LayerMask;
use vocs::nibbles::{u4, BulkNibbles, ChunkNibbles};
use vocs::position::{dir, Offset, ChunkPosition, GlobalColumnPosition, GlobalSectorPosition, LayerPosition};
use vocs::unpacked::Layer;
use vocs::view::{Directional, SplitDirectional};
use vocs::world::sector::Sector;
use vocs::world::shared::{NoPack, SharedSector, SharedWorld};
use vocs::world::world::World;

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
		let mut queue = ChunkQueue::new();

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

			let chunk_position = ChunkPosition::from_layer(y as u8, position);

			sector_queue.lock().unwrap().enqueue_spills(chunk_position, queue.reset_spills());
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

		while let Some((position, incomplete)) = sector_queue.pop_first() {
			chunk_operations += 1;

			let blocks = block_sector[position].as_ref().unwrap();
			let column_heightmap = &heightmaps[position.layer()];

			let heightmap = column_heightmap.slice(u4::new(position.y()));

			let mut queue = complete_chunk(position, blocks, sky_light, sky_light_neighbors, incomplete, &heightmap);

			sector_queue.enqueue_spills(position, queue.reset_spills());
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
	heightmap: &ChunkHeightMap) -> ChunkQueue {

	// TODO: Cache these things!
	let lighting_info = lighting_info();
	let empty_lighting = ChunkNibbles::default();
	let mut queue = ChunkQueue::new();

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

fn spill<P, M>(sector: &mut Sector<ChunkMask>, layer: Layer<Option<LayerMask>>, position: P, mut merge: M)
	where P: Fn(LayerPosition) -> ChunkPosition, M: FnMut(&mut ChunkMask, LayerMask) {

	for (index, spilled) in layer.into_inner().into_vec().drain(..).enumerate() {
		let spilled: Option<LayerMask> = spilled;

		let spilled = match spilled {
			Some(mask) => mask,
			None => continue
		};

		let layer_position = LayerPosition::from_zx(index as u8);
		let chunk_position = position(layer_position);

		merge(sector.get_or_create_mut(chunk_position), spilled);
	}
}

fn process_sector_spills(spills: HashMap<GlobalSectorPosition, SectorSpills>) -> World<ChunkMask> {
	let mut world: World<ChunkMask> = World::new();

	for (position, spills) in spills {
		let spills = spills.0.split();
		
		spill(
			world.get_or_create_sector_mut(GlobalSectorPosition::new(position.x() + 1, position.z())), 
			spills.plus_x, 
			|layer_position| ChunkPosition::new(0, layer_position.x(), layer_position.z()),
			|mask, layer| mask.layer_zy_mut(0).combine(&layer)
		);

		spill(
			world.get_or_create_sector_mut(GlobalSectorPosition::new(position.x() - 1, position.z())), 
			spills.minus_x, 
			|layer_position| ChunkPosition::new(15, layer_position.x(), layer_position.z()),
			|mask, layer| mask.layer_zy_mut(15).combine(&layer)
		);

		spill(
			world.get_or_create_sector_mut(GlobalSectorPosition::new(position.x(), position.z() + 1)), 
			spills.plus_z, 
			|layer_position| ChunkPosition::new(layer_position.x(), layer_position.z(), 0),
			|mask, layer| mask.layer_yx_mut(0).combine(&layer)
		);

		spill(
			world.get_or_create_sector_mut(GlobalSectorPosition::new(position.x(), position.z() - 1)), 
			spills.minus_z, 
			|layer_position| ChunkPosition::new(layer_position.x(), layer_position.z(), 15),
			|mask, layer| mask.layer_yx_mut(15).combine(&layer)
		);
	}

	world
}

pub fn compute_skylight(world: &World<ChunkIndexed<Block>>) -> (SharedWorld<NoPack<ChunkNibbles>>, HashMap<GlobalColumnPosition, ColumnHeightMap>) {
	let empty_sector: SharedSector<NoPack<ChunkNibbles>> = SharedSector::new();

	let mut sky_light: SharedWorld<NoPack<ChunkNibbles>> = SharedWorld::new();
	let heightmaps: Mutex<HashMap<GlobalSectorPosition, Layer<ColumnHeightMap>>> = Mutex::new(HashMap::new());
	let spills: Mutex<HashMap<GlobalSectorPosition, SectorSpills>> = Mutex::new(HashMap::new());

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

		let sector_spills = sector_queue.reset_spills();

		spills.lock().unwrap().insert(position, sector_spills);
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
	let queues = Mutex::new(process_sector_spills(spills.into_inner().unwrap()).into_sectors());

	let complete_sector = |position: GlobalSectorPosition| {
		let sector_mask = match queues.lock().unwrap().remove(&position) {
			Some(mask) => mask,
			None => {
				println!("Skipping sky light completion for ({}, {}), nothing queued", position.x(), position.z());

				return;
			}
		};

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

		let mut sector_queue = SectorQueue::new();
		sector_queue.reset_from_mask(sector_mask);

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
