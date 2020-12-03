use crate::heightmap::ColumnHeightMap;
use crate::light::Lighting;
use crate::queue::{CubeQueue, SectorQueue, WorldQueue};
use crate::sources::{LightSources, SkyLightSources};

use rayon::iter::ParallelBridge;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use vocs::indexed::{IndexedCube, Target};
use vocs::mask::BitCube;
use vocs::nibbles::{u4, NibbleArray, NibbleCube};
use vocs::position::{dir, CubePosition, GlobalSectorPosition, Offset};
use vocs::unpacked::Layer;
use vocs::view::{Directional, SplitDirectional};
use vocs::world::sector::Sector;
use vocs::world::shared::{NoPack, SharedSector, SharedWorld};
use vocs::world::world::World;

// TODO: This whole file should be split up / refactored at some point

fn initial_sector<'a, B, F>(
	block_sector: &'a Sector<IndexedCube<B>>, sky_light: &SharedSector<NoPack<NibbleCube>>,
	heightmaps: &Layer<ColumnHeightMap>, opacities: &'a F,
) -> SectorQueue
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> u4 + Sync,
{
	let empty_lighting = NibbleCube::default();
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
		.map(|(index, chunk)| {
			(CubePosition::from_yzx(index as u16), chunk.as_ref().expect("TODO").freeze())
		})
		.for_each(|(position, (blocks, palette))| {
			let mut opacity = NibbleArray::new(palette.len());

			for (index, value) in palette.iter().enumerate() {
				let opacity_value = value.as_ref().map(opacities).unwrap_or(u4::new(15));

				opacity.set(index, opacity_value);
			}

			let column_heightmap = &heightmaps[position.layer()];
			let chunk_heightmap = column_heightmap.slice(u4::new(position.y()));
			let sources = SkyLightSources::new(chunk_heightmap);

			let mut light_data = NibbleCube::default();

			let mut light = Lighting::new(&mut light_data, empty_neighbors, sources, opacity);

			// TODO: Reuse this!
			let mut queue = CubeQueue::new();
			light.initial(&mut queue);
			light.apply(blocks, &mut queue);

			sector_queue.lock().unwrap().enqueue_spills(position, queue.reset_spills());
			sky_light.set(position, NoPack(light_data));
		});

	sector_queue.into_inner().unwrap()
}

fn full_sector<'a, B, F, S, SF>(
	block_sector: &'a Sector<IndexedCube<B>>, light: &SharedSector<NoPack<NibbleCube>>,
	light_neighbors: Directional<&SharedSector<NoPack<NibbleCube>>>,
	sector_queue: &mut SectorQueue, sector_sources: &SF, opacities: &'a F,
) -> (u32, u32)
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> u4 + Sync,
	S: LightSources,
	SF: Fn(CubePosition) -> S
{
	let mut iterations = 0;
	let mut chunk_operations = 0;

	while sector_queue.flip() {
		iterations += 1;

		while let Some((position, incomplete)) = sector_queue.pop_first() {
			chunk_operations += 1;

			let blocks = block_sector[position].as_ref().unwrap();
			let sources = sector_sources(position);

			let mut queue = complete_chunk(
				position,
				blocks,
				light,
				light_neighbors,
				incomplete,
				sources,
				opacities,
			);

			sector_queue.enqueue_spills(position, queue.reset_spills());
		}
	}

	(iterations, chunk_operations)
}

fn complete_chunk<'a, B, F, S>(
	position: CubePosition, blocks: &'a IndexedCube<B>,
	light: &SharedSector<NoPack<NibbleCube>>,
	light_neighbors: Directional<&SharedSector<NoPack<NibbleCube>>>, incomplete: BitCube,
	sources: S, opacities: &'a F,
) -> CubeQueue
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> u4 + Sync,
	S: LightSources,
{
	// TODO: Cache these things!
	let empty_lighting = NibbleCube::default();
	let mut queue = CubeQueue::new();

	let (blocks, palette) = blocks.freeze();

	let mut opacity = NibbleArray::new(palette.len());

	for (index, value) in palette.iter().enumerate() {
		let opacity_value = value.as_ref().map(opacities).unwrap_or(u4::new(15));

		opacity.set(index, opacity_value);
	}

	let mut central = light.get_or_create(position);
	let locks = SplitDirectional {
		up: position.offset(dir::Up).map(|position| light[position].read()).unwrap_or_else(
			|| light_neighbors[dir::Up][position.offset_wrapping(dir::Up)].read(),
		),
		down: position.offset(dir::Down).map(|position| light[position].read()).unwrap_or_else(
			|| light_neighbors[dir::Down][position.offset_wrapping(dir::Down)].read(),
		),
		plus_x: position
			.offset(dir::PlusX)
			.map(|position| light[position].read())
			.unwrap_or_else(|| {
				light_neighbors[dir::PlusX][position.offset_wrapping(dir::PlusX)].read()
			}),
		minus_x: position
			.offset(dir::MinusX)
			.map(|position| light[position].read())
			.unwrap_or_else(|| {
				light_neighbors[dir::MinusX][position.offset_wrapping(dir::MinusX)].read()
			}),
		plus_z: position
			.offset(dir::PlusZ)
			.map(|position| light[position].read())
			.unwrap_or_else(|| {
				light_neighbors[dir::PlusZ][position.offset_wrapping(dir::PlusZ)].read()
			}),
		minus_z: position
			.offset(dir::MinusZ)
			.map(|position| light[position].read())
			.unwrap_or_else(|| {
				light_neighbors[dir::MinusZ][position.offset_wrapping(dir::MinusZ)].read()
			}),
	};

	let neighbors = SplitDirectional {
		up: locks.up.as_ref().map(|chunk| &chunk.0).unwrap_or(&empty_lighting),
		down: locks.down.as_ref().map(|chunk| &chunk.0).unwrap_or(&empty_lighting),
		plus_x: locks.plus_x.as_ref().map(|chunk| &chunk.0).unwrap_or(&empty_lighting),
		minus_x: locks.minus_x.as_ref().map(|chunk| &chunk.0).unwrap_or(&empty_lighting),
		plus_z: locks.plus_z.as_ref().map(|chunk| &chunk.0).unwrap_or(&empty_lighting),
		minus_z: locks.minus_z.as_ref().map(|chunk| &chunk.0).unwrap_or(&empty_lighting),
	};

	let mut light_operation = Lighting::new(&mut central, Directional::combine(neighbors), sources, opacity);

	queue.reset_from_mask(incomplete);
	light_operation.apply(blocks, &mut queue);

	queue
}

pub trait SkyLightTraces {
	fn initial_sector(&self, position: GlobalSectorPosition, duration: Duration);
	fn initial_full_sector(
		&self, position: GlobalSectorPosition, iterations: u32, chunk_operations: u32,
		duration: Duration,
	);
	fn complete_sector(
		&self, position: GlobalSectorPosition, iteration: u32, inner_iterations: u32,
		chunk_operations: u32, duration: Duration,
	);
}

pub struct PrintTraces;

impl PrintTraces {
	fn us(duration: Duration) -> u64 {
		(duration.as_secs() * 1000000) + ((duration.subsec_nanos() / 1000) as u64)
	}
}

impl SkyLightTraces for PrintTraces {
	fn initial_sector(&self, position: GlobalSectorPosition, duration: Duration) {
		let us = Self::us(duration);

		println!(
			"Initial sky lighting for ({}, {}) done in {}us ({}us per column)",
			position.x(),
			position.z(),
			us,
			us / 256
		);
	}

	fn initial_full_sector(
		&self, position: GlobalSectorPosition, iterations: u32, chunk_operations: u32,
		duration: Duration,
	) {
		let us = Self::us(duration);

		println!("Inner full sky lighting for ({}, {}) done in {}us ({}us per column): {} iterations, {} post-initial chunk light operations", position.x(), position.z(), us, us / 256, iterations, chunk_operations);
	}

	fn complete_sector(
		&self, position: GlobalSectorPosition, iteration: u32, inner_iterations: u32,
		chunk_operations: u32, duration: Duration,
	) {
		let us = Self::us(duration);

		println!("Full sky lighting for ({}, {}) [iteration {}] done in {}us ({}us per column): {} iterations, {} post-initial chunk light operations", position.x(), position.z(), iteration, us, us / 256, inner_iterations, chunk_operations);
	}
}

pub struct IgnoreTraces;

impl SkyLightTraces for IgnoreTraces {
	fn initial_sector(&self, _: GlobalSectorPosition, _: Duration) {}
	fn initial_full_sector(&self, _: GlobalSectorPosition, _: u32, _: u32, _: Duration) {}
	fn complete_sector(&self, _: GlobalSectorPosition, _: u32, _: u32, _: u32, _: Duration) {}
}

pub fn compute_world_skylight<'a, B, F, T>(
	world: &'a World<IndexedCube<B>>,
	heightmaps: &HashMap<GlobalSectorPosition, Layer<ColumnHeightMap>>, opacities: &'a F,
	tracer: &T,
) -> SharedWorld<NoPack<NibbleCube>>
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> u4 + Sync,
	T: SkyLightTraces + Sync,
{
	let empty_sector: SharedSector<NoPack<NibbleCube>> = SharedSector::new();
	let empty_sky_light_neighbors = Directional::combine(SplitDirectional {
		minus_x: &empty_sector,
		plus_x: &empty_sector,
		minus_z: &empty_sector,
		plus_z: &empty_sector,
		down: &empty_sector,
		up: &empty_sector,
	});

	let mut sky_light: SharedWorld<NoPack<NibbleCube>> = SharedWorld::new();
	let world_queue = Mutex::new(WorldQueue::new());

	world.sectors().map(|entry| *entry.0).for_each(|position| {
		sky_light.get_or_create_sector_mut(position);
	});

	world.sectors().par_bridge().for_each(|(&position, block_sector)| {
		let initial_start = Instant::now();

		let sky_light = sky_light.get_sector(position).unwrap();
		let sector_heightmaps = heightmaps.get(&position).unwrap();
		let sector_sources = &|position: CubePosition| {
			let column_heightmap = &sector_heightmaps[position.layer()];
	
			let heightmap = column_heightmap.slice(u4::new(position.y()));
			SkyLightSources::new(heightmap)
		};

		let mut sector_queue =
			initial_sector(block_sector, sky_light, sector_heightmaps, opacities);

		let inner_start = Instant::now();
		tracer.initial_sector(position, inner_start.duration_since(initial_start));

		let (iterations, chunk_operations) = full_sector(
			block_sector,
			sky_light,
			empty_sky_light_neighbors,
			&mut sector_queue,
			sector_sources,
			opacities,
		);

		let sector_spills = sector_queue.reset_spills();

		world_queue.lock().unwrap().enqueue_spills(position, sector_spills);

		tracer.initial_full_sector(
			position,
			iterations,
			chunk_operations,
			Instant::now().duration_since(inner_start),
		);
	});

	let mut iterations = 0;

	while let Some(world_masks) = {
		let mask = world_queue.lock().unwrap().flip();
		mask
	} {
		iterations += 1;

		let complete_sector =
			|(position, sector_mask): (GlobalSectorPosition, Sector<BitCube>)| {
				let block_sector = match world.get_sector(position) {
					Some(sector) => sector,
					None => return,
				};

				let full_start = Instant::now();

				let sky_light_center = sky_light.get_sector(position).unwrap();

				let sky_light_neighbors = Directional::combine(SplitDirectional {
					minus_x: sky_light
						.get_sector(GlobalSectorPosition::new(position.x() - 1, position.z()))
						.unwrap_or(&empty_sector),
					plus_x: sky_light
						.get_sector(GlobalSectorPosition::new(position.x() + 1, position.z()))
						.unwrap_or(&empty_sector),
					minus_z: sky_light
						.get_sector(GlobalSectorPosition::new(position.x(), position.z() - 1))
						.unwrap_or(&empty_sector),
					plus_z: sky_light
						.get_sector(GlobalSectorPosition::new(position.x(), position.z() + 1))
						.unwrap_or(&empty_sector),
					down: &empty_sector,
					up: &empty_sector,
				});

				let mut sector_queue = SectorQueue::new();
				sector_queue.reset_from_mask(sector_mask);

				let sector_heightmaps = heightmaps.get(&position).unwrap();
				let sector_sources = &|position: CubePosition| {
					let column_heightmap = &sector_heightmaps[position.layer()];
			
					let heightmap = column_heightmap.slice(u4::new(position.y()));
					SkyLightSources::new(heightmap)
				};

				let (inner_iterations, chunk_operations) = full_sector(
					block_sector,
					sky_light_center,
					sky_light_neighbors,
					&mut sector_queue,
					sector_sources,
					opacities,
				);

				world_queue.lock().unwrap().enqueue_spills(position, sector_queue.reset_spills());

				tracer.complete_sector(
					position,
					iterations,
					inner_iterations,
					chunk_operations,
					Instant::now().duration_since(full_start),
				);
			};

		world_masks.into_par_iter().for_each(complete_sector);
	}

	sky_light
}
