use lumis::heightmap::{ChunkHeightMap, ColumnHeightMap};
use lumis::light::Lighting;
use lumis::sources::SkyLightSources;
use lumis::queue::{ChunkQueue, SectorQueue, SectorSpills};

use rayon::iter::ParallelBridge;
use rayon::prelude::{ParallelIterator, IntoParallelIterator};

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use vocs::indexed::{ChunkIndexed, Target};
use vocs::mask::ChunkMask;
use vocs::mask::LayerMask;
use vocs::nibbles::{u4, BulkNibbles, ChunkNibbles};
use vocs::position::{dir, Offset, ChunkPosition, GlobalSectorPosition, LayerPosition};
use vocs::unpacked::Layer;
use vocs::view::{Directional, SplitDirectional};
use vocs::world::sector::Sector;
use vocs::world::shared::{NoPack, SharedSector, SharedWorld};
use vocs::world::world::World;

fn initial_sector<'a, B, F>(
	block_sector: &'a Sector<ChunkIndexed<B>>, sky_light: &SharedSector<NoPack<ChunkNibbles>>,
	heightmaps: &Layer<ColumnHeightMap>, opacities: &'a F
) -> SectorQueue
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> u4 + Sync,
{
	let empty_lighting = ChunkNibbles::default();
	let empty_neighbors = Directional::combine(SplitDirectional {
		minus_x: &empty_lighting,
		plus_x: &empty_lighting,
		minus_z: &empty_lighting,
		plus_z: &empty_lighting,
		down: &empty_lighting,
		up: &empty_lighting,
	});

	let sector_queue = Mutex::new(SectorQueue::new());

	block_sector
		.iter()
		.enumerate()
		.par_bridge()
		.map(|(index, chunk)| (ChunkPosition::from_yzx(index as u16), chunk.as_ref().expect("TODO").freeze()))
		.for_each(|(position, (blocks, palette))| {
			let mut opacity = BulkNibbles::new(palette.len());

			for (index, value) in palette.iter().enumerate() {
				let opacity_value = value.as_ref().map(opacities).unwrap_or(u4::new(15));
				
				opacity.set(index, opacity_value);
			}

			let column_heightmap = &heightmaps[position.layer()];
			let chunk_heightmap = column_heightmap.slice(u4::new(position.y()));
			let sources = SkyLightSources::new(&chunk_heightmap);

			let mut light_data = ChunkNibbles::default();

			let mut light = Lighting::new(&mut light_data, empty_neighbors, sources, opacity);

			// TODO: Reuse this!
			let mut queue = ChunkQueue::new();
			light.initial(&mut queue);
			light.apply(blocks, &mut queue);

			sector_queue.lock().unwrap().enqueue_spills(position, queue.reset_spills());
			sky_light.set(position, NoPack(light_data));
		});

	sector_queue.into_inner().unwrap()
}

fn full_sector<'a, B, F>(
	block_sector: &'a Sector<ChunkIndexed<B>>, 
	sky_light: &SharedSector<NoPack<ChunkNibbles>>, 
	sky_light_neighbors: Directional<&SharedSector<NoPack<ChunkNibbles>>>, 
	sector_queue: &mut SectorQueue, 
	heightmaps: &Layer<ColumnHeightMap>, 
	opacities: &'a F
) -> (u32, u32)
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> u4 + Sync,
{

	let mut iterations = 0;
	let mut chunk_operations = 0;

	while sector_queue.flip() {
		iterations += 1;

		while let Some((position, incomplete)) = sector_queue.pop_first() {
			chunk_operations += 1;

			let blocks = block_sector[position].as_ref().unwrap();
			let column_heightmap = &heightmaps[position.layer()];

			let heightmap = column_heightmap.slice(u4::new(position.y()));

			let mut queue = complete_chunk(position, blocks, sky_light, sky_light_neighbors, incomplete, &heightmap, opacities);

			sector_queue.enqueue_spills(position, queue.reset_spills());
		}
	}

	(iterations, chunk_operations)
}

fn complete_chunk<'a, B, F>(
	position: ChunkPosition, 
	blocks: &'a ChunkIndexed<B>, 
	sky_light: &SharedSector<NoPack<ChunkNibbles>>, 
	sky_light_neighbors: Directional<&SharedSector<NoPack<ChunkNibbles>>>, 
	incomplete: ChunkMask, 
	heightmap: &ChunkHeightMap, 
	opacities: &'a F
) -> ChunkQueue
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> u4 + Sync,
{

	// TODO: Cache these things!
	let empty_lighting = ChunkNibbles::default();
	let mut queue = ChunkQueue::new();

	let (blocks, palette) = blocks.freeze();

	let mut opacity = BulkNibbles::new(palette.len());

	for (index, value) in palette.iter().enumerate() {
		let opacity_value = value.as_ref().map(opacities).unwrap_or(u4::new(15));

		opacity.set(index, opacity_value);
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

#[derive(Copy, Clone)]
enum Phase {
	Odd,
	Even
}

impl Phase {
	fn from_position(position: GlobalSectorPosition) -> Self {
		let is_odd = position.x().wrapping_add(position.z()) & 1 == 1;

		if is_odd {
			Phase::Odd
		} else {
			Phase::Even
		}
	}

	fn next(self) -> Self {
		match self {
			Phase::Odd => Phase::Even,
			Phase::Even => Phase::Odd
		}
	}
}

struct WorldQueue {
	odd: HashMap<GlobalSectorPosition, Sector<ChunkMask>>,
	even: HashMap<GlobalSectorPosition, Sector<ChunkMask>>,
	phase: Phase
}

impl WorldQueue {
	pub fn new() -> WorldQueue {
		WorldQueue {
			odd: HashMap::new(),
			even: HashMap::new(),
			phase: Phase::Odd
		}
	}

	pub fn enqueue_spills(&mut self, position: GlobalSectorPosition, spills: SectorSpills) {
		let spills = spills.0.split();
		
		self.spill(
			GlobalSectorPosition::new(position.x() + 1, position.z()), 
			spills.plus_x, 
			|layer_position| ChunkPosition::new(0, layer_position.x(), layer_position.z()),
			|mask, layer| mask.layer_zy_mut(0).combine(&layer)
		);

		self.spill(
			GlobalSectorPosition::new(position.x() - 1, position.z()), 
			spills.minus_x, 
			|layer_position| ChunkPosition::new(15, layer_position.x(), layer_position.z()),
			|mask, layer| mask.layer_zy_mut(15).combine(&layer)
		);

		self.spill(
			GlobalSectorPosition::new(position.x(), position.z() + 1),
			spills.plus_z, 
			|layer_position| ChunkPosition::new(layer_position.x(), layer_position.z(), 0),
			|mask, layer| mask.layer_yx_mut(0).combine(&layer)
		);

		self.spill(
			GlobalSectorPosition::new(position.x(), position.z() - 1), 
			spills.minus_z, 
			|layer_position| ChunkPosition::new(layer_position.x(), layer_position.z(), 15),
			|mask, layer| mask.layer_yx_mut(15).combine(&layer)
		);
	}

	fn sector_masks(&mut self, position: GlobalSectorPosition) -> &mut Sector<ChunkMask> {
		match Phase::from_position(position) {
			Phase::Odd => &mut self.odd,
			Phase::Even => &mut self.even
		}.entry(position).or_insert_with(Sector::new)
	}

	fn spill<P, M>(&mut self, origin: GlobalSectorPosition, layer: Layer<Option<LayerMask>>, position: P, mut merge: M)
		where P: Fn(LayerPosition) -> ChunkPosition, M: FnMut(&mut ChunkMask, LayerMask) {

		use vocs::component::LayerStorage;

		for (index, spilled) in layer.into_inner().into_vec().drain(..).enumerate() {
			let spilled: Option<LayerMask> = spilled;

			let spilled = match spilled {
				Some(mask) => mask,
				None => continue
			};

			if spilled.is_filled(false) {
				continue;
			}

			let layer_position = LayerPosition::from_zx(index as u8);
			let chunk_position = position(layer_position);

			// TODO: Don't repeatedly perform hashmap lookups
			let sector = self.sector_masks(origin);

			merge(sector.get_or_create_mut(chunk_position), spilled);
		}
	}

	pub fn flip(&mut self) -> Option<HashMap<GlobalSectorPosition, Sector<ChunkMask>>> {
		match (self.even.is_empty(), self.odd.is_empty()) {
			(true, true) => {
				self.phase = Phase::Odd;

				None
			},
			(false, true) => {
				self.phase = Phase::Even;

				Some(std::mem::replace(&mut self.even, HashMap::new()))
			},
			(true, false) => {
				self.phase = Phase::Odd;

				Some(std::mem::replace(&mut self.odd, HashMap::new()))
			},
			(false, false) => {
				self.phase = self.phase.next();

				Some(std::mem::replace(match self.phase {
					Phase::Odd => &mut self.odd,
					Phase::Even => &mut self.even
				}, HashMap::new()))
			}
		}
	}
}

pub fn compute_skylight<'a, B, F>(
	world: &'a World<ChunkIndexed<B>>, heightmaps: &HashMap<GlobalSectorPosition, Layer<ColumnHeightMap>>,
	opacities: &'a F
) -> SharedWorld<NoPack<ChunkNibbles>>
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> u4 + Sync,
{
	let empty_sector: SharedSector<NoPack<ChunkNibbles>> = SharedSector::new();
	let empty_sky_light_neighbors = Directional::combine(SplitDirectional {
		minus_x: &empty_sector,
		plus_x: &empty_sector,
		minus_z: &empty_sector,
		plus_z: &empty_sector,
		down: &empty_sector,
		up: &empty_sector
	});

	let mut sky_light: SharedWorld<NoPack<ChunkNibbles>> = SharedWorld::new();
	let world_queue = Mutex::new(WorldQueue::new());

	world.sectors().map(|entry| *entry.0).for_each(|position| {
		sky_light.get_or_create_sector_mut(position);
	});

	world.sectors().par_bridge().for_each(|(&position, block_sector)| {
		let initial_start = Instant::now();

		println!("Performing initial lighting for sector ({}, {})", position.x(), position.z());

		let sky_light = sky_light.get_sector(position).unwrap();
		let sector_heightmaps = heightmaps.get(&position).unwrap();

		let mut sector_queue = initial_sector(block_sector, sky_light, sector_heightmaps, opacities);

		{
			let end = ::std::time::Instant::now();
			let time = end.duration_since(initial_start);
	
			let secs = time.as_secs();
			let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);
	
			println!("Initial sky lighting for ({}, {}) done in {}us ({}us per column)", position.x(), position.z(), us, us / 256);
		}

		println!("Performing inner full sky lighting for sector ({}, {})", position.x(), position.z());
		let inner_start = Instant::now();

		let (iterations, chunk_operations) = full_sector(block_sector, sky_light, empty_sky_light_neighbors, &mut sector_queue, &sector_heightmaps, opacities);

		let sector_spills = sector_queue.reset_spills();

		world_queue.lock().unwrap().enqueue_spills(position, sector_spills);

		{
			let end = ::std::time::Instant::now();
			let time = end.duration_since(inner_start);
	
			let secs = time.as_secs();
			let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);
	
			println!("Inner full sky lighting done in {}us ({}us per column): {} iterations, {} post-initial chunk light operations", us, us / 256, iterations, chunk_operations);
		}
	});

	let mut iterations = 0;

	while let Some(world_masks) = {let mask = world_queue.lock().unwrap().flip(); mask} {
		iterations += 1;

		let complete_sector = |(position, sector_mask): (GlobalSectorPosition, Sector<ChunkMask>)| {
			let block_sector = match world.get_sector(position) {
				Some(sector) => sector,
				None => return
			};

			let full_start = Instant::now();
			println!("Performing full sky lighting for sector ({}, {}) [iteration: {}]", position.x(), position.z(), iterations);
	
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
	
			let (iterations, chunk_operations) = full_sector(block_sector, sky_light_center, sky_light_neighbors, &mut sector_queue, sector_heightmaps, opacities);
	
			world_queue.lock().unwrap().enqueue_spills(position, sector_queue.reset_spills());
	
			{
				let end = ::std::time::Instant::now();
				let time = end.duration_since(full_start);
		
				let secs = time.as_secs();
				let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);
		
				println!("Full sky lighting done in {}us ({}us per column): {} iterations, {} post-initial chunk light operations", us, us / 256, iterations, chunk_operations);
			}
		};

		world_masks.into_par_iter().for_each(complete_sector);
	}

	sky_light
}
