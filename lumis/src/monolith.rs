use crate::heightmap::ColumnHeightMap;
use crate::light::Lighting;
use crate::queue::{CubeQueue, SectorQueue, WorldQueue};
use crate::sources::{LightSources, BlockLightSources, EmissionPalette, SkyLightSources};
use crate::PackedNibbleCube;

use rayon::iter::ParallelBridge;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use vocs::indexed::{IndexedCube, Target};
use vocs::mask::BitCube;
use vocs::nibbles::{u4, NibbleArray};
use vocs::position::{dir, CubePosition, GlobalSectorPosition, Offset};
use vocs::unpacked::Layer;
use vocs::view::{Directional, SplitDirectional};
use vocs::world::sector::Sector;
use vocs::world::shared::{NoPack, SharedSector, SharedWorld};
use vocs::world::world::World;

// TODO: This whole file should be split up / refactored at some point

fn initial_sector<B, F, S>(
	block_sector: &Sector<IndexedCube<B>>, light: &SharedSector<NoPack<PackedNibbleCube>>,
	sector_sources: &S::SectorSources, emission_palette: &S::EmissionPalette, opacities: &F,
) -> SectorQueue
where
	B: Target + Send + Sync,
	F: Fn(&B) -> u4 + Sync,
	S: LightSources,
{
	let empty_neighbors = Directional::splat(&PackedNibbleCube::EntirelyDark);

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

			let sources = S::chunk_sources(sector_sources, emission_palette, position);

			// TODO: Reuse this!
			let mut queue = CubeQueue::new();
			let mut light_data = sources.initial(blocks, queue.mask_mut());

			if !queue.mask_mut().primary.empty() {
				// If there's anything queued, it's going to require unpacking the light data anyways
				light_data.unpack_in_place();

				let mut light_operation = Lighting::new(&mut light_data, empty_neighbors, sources, opacity);
				light_operation.apply(blocks, &mut queue);
			}

			sector_queue.lock().unwrap().enqueue_spills(position, queue.reset_spills());
			light.set(position, NoPack(light_data));
		});

	sector_queue.into_inner().unwrap()
}

fn full_sector<B, F, S>(
	block_sector: &Sector<IndexedCube<B>>, light: &SharedSector<NoPack<PackedNibbleCube>>,
	light_neighbors: Directional<&SharedSector<NoPack<PackedNibbleCube>>>,
	sector_queue: &mut SectorQueue, sector_sources: &S::SectorSources, emission_palette: &S::EmissionPalette, opacities: &F,
) -> (u32, u32)
where
	B: Target + Send + Sync,
	F: Fn(&B) -> u4 + Sync,
	S: LightSources,
{
	let mut iterations = 0;
	let mut chunk_operations = 0;

	while sector_queue.flip() {
		iterations += 1;

		while let Some((position, incomplete)) = sector_queue.pop_first() {
			chunk_operations += 1;

			let blocks = block_sector[position].as_ref().unwrap();
			let sources = S::chunk_sources(sector_sources, emission_palette, position);

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

fn complete_chunk<B, F, S>(
	position: CubePosition, blocks: &IndexedCube<B>,
	light: &SharedSector<NoPack<PackedNibbleCube>>,
	light_neighbors: Directional<&SharedSector<NoPack<PackedNibbleCube>>>, incomplete: BitCube,
	sources: S, opacities: &F,
) -> CubeQueue
where
	B: Target + Send + Sync,
	F: Fn(&B) -> u4 + Sync,
	S: LightSources,
{
	// TODO: Cache these things!
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

	let neighbors = locks.as_ref().map(|guard| {
		guard.as_ref().map(|chunk| &chunk.0).unwrap_or(&PackedNibbleCube::EntirelyDark)
	});

	let mut light_operation = Lighting::new(&mut central, Directional::combine(neighbors), sources, opacity);

	queue.reset_from_mask(incomplete);
	light_operation.apply(blocks, &mut queue);

	queue
}

pub trait LightTraces {
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

pub struct PrintTraces(pub &'static str);

impl PrintTraces {
	fn us(duration: Duration) -> u64 {
		(duration.as_secs() * 1000000) + ((duration.subsec_nanos() / 1000) as u64)
	}
}

impl LightTraces for PrintTraces {
	fn initial_sector(&self, position: GlobalSectorPosition, duration: Duration) {
		let us = Self::us(duration);

		println!(
			"Initial {} lighting for ({}, {}) done in {}us ({}us per column)",
			self.0,
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

		println!("Inner full {} lighting for ({}, {}) done in {}us ({}us per column): {} iterations, {} post-initial chunk light operations", self.0, position.x(), position.z(), us, us / 256, iterations, chunk_operations);
	}

	fn complete_sector(
		&self, position: GlobalSectorPosition, iteration: u32, inner_iterations: u32,
		chunk_operations: u32, duration: Duration,
	) {
		let us = Self::us(duration);

		println!("Full {} lighting for ({}, {}) [iteration {}] done in {}us ({}us per column): {} iterations, {} post-initial chunk light operations", self.0, position.x(), position.z(), iteration, us, us / 256, inner_iterations, chunk_operations);
	}
}

pub struct IgnoreTraces;

impl LightTraces for IgnoreTraces {
	fn initial_sector(&self, _: GlobalSectorPosition, _: Duration) {}
	fn initial_full_sector(&self, _: GlobalSectorPosition, _: u32, _: u32, _: Duration) {}
	fn complete_sector(&self, _: GlobalSectorPosition, _: u32, _: u32, _: u32, _: Duration) {}
}

pub fn compute_world_light<B, F, T, S>(
	world: &World<IndexedCube<B>>,
	opacities: &F,
	world_sources: &S::WorldSources,
	emission_palette: &S::EmissionPalette,
	tracer: &T,
) -> SharedWorld<NoPack<PackedNibbleCube>>
where
	B: Target + Send + Sync,
	F: Fn(&B) -> u4 + Sync,
	T: LightTraces + Sync,
	S: LightSources,
{
	let empty_sector: SharedSector<NoPack<PackedNibbleCube>> = SharedSector::new();
	let empty_light_neighbors = Directional::splat(&empty_sector);

	let mut sky_light: SharedWorld<NoPack<PackedNibbleCube>> = SharedWorld::new();
	let world_queue = Mutex::new(WorldQueue::new());

	world.sectors().map(|entry| *entry.0).for_each(|position| {
		sky_light.get_or_create_sector_mut(position);
	});

	world.sectors().par_bridge().for_each(|(&position, block_sector)| {
		let initial_start = Instant::now();

		let sky_light = sky_light.get_sector(position).unwrap();
		let sector_sources = S::sector_sources(world_sources, position);

		let mut sector_queue =
			initial_sector::<B, F, S>(block_sector, sky_light, &sector_sources, emission_palette, opacities);

		let inner_start = Instant::now();
		tracer.initial_sector(position, inner_start.duration_since(initial_start));

		let (iterations, chunk_operations) = full_sector::<B, F, S>(
			block_sector,
			sky_light,
			empty_light_neighbors,
			&mut sector_queue,
			sector_sources,
			emission_palette,
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
				let sector_sources = S::sector_sources(world_sources, position);

				let (inner_iterations, chunk_operations) = full_sector::<B, F, S>(
					block_sector,
					sky_light_center,
					sky_light_neighbors,
					&mut sector_queue,
					sector_sources,
					emission_palette,
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

pub fn compute_world_skylight<B, F, T>(
	world: &World<IndexedCube<B>>,
	heightmaps: &HashMap<GlobalSectorPosition, Layer<ColumnHeightMap>>,
	opacities: &F,
	tracer: &T,
) -> SharedWorld<NoPack<PackedNibbleCube>>
where
	B: Target + Send + Sync,
	F: Fn(&B) -> u4 + Sync,
	T: LightTraces + Sync,
{
	let world_sources = heightmaps;

	compute_world_light::< _, _, _, SkyLightSources>(world, opacities, world_sources, &(), tracer)
}

pub fn compute_world_blocklight<B, F, E, T>(
	world: &World<IndexedCube<B>>,
	opacities: &F,
	emissions: &E,
	tracer: &T,
) -> SharedWorld<NoPack<PackedNibbleCube>>
where
	B: Target + Send + Sync,
	F: Fn(&B) -> u4 + Sync,
	E: EmissionPalette<B>,
	T: LightTraces + Sync,
{
	compute_world_light::< _, _, _, BlockLightSources<B, E>>(world, opacities, world, emissions, tracer)
}
