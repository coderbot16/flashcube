use bit_vec::BitVec;

use i73_base::Block;

// use rs25::dynamics::light::{HeightMapBuilder, Lighting, SkyLightSources};
// use rs25::dynamics::queue::Queue;

use lumis::heightmap::{ChunkHeightMap, ColumnHeightMap, HeightMapBuilder};
use lumis::light::Lighting;
use lumis::sources::SkyLightSources;
use lumis::queue::Queue;

use std::collections::HashMap;
use std::ops::IndexMut;
use std::time::Instant;

use vocs::component::LayerStorage;
use vocs::indexed::ChunkIndexed;
use vocs::mask::ChunkMask;
use vocs::mask::LayerMask;
use vocs::nibbles::{u4, BulkNibbles, ChunkNibbles};
use vocs::position::{dir, Offset, ChunkPosition, GlobalChunkPosition, GlobalColumnPosition, GlobalSectorPosition, LayerPosition};
use vocs::view::{Directional, SplitDirectional};
use vocs::world::sector::Sector;
use vocs::world::shared::{NoPack, SharedSector, SharedWorld, Guard};
use vocs::world::world::World;

struct SectorSpills<'a> {
	spills: &'a SharedSector<NoPack<ChunkMask>>,
	neighbors: Directional<&'a SharedSector<NoPack<ChunkMask>>>
}

impl<'a> SectorSpills<'a> {
	fn spill_out(&mut self, origin: ChunkPosition, spills: Directional<LayerMask>) {
		if !spills[dir::Up].is_filled(false) {
			self.mask(origin, dir::Up, origin.with_y(0)).layer_zx_mut(0).combine(&spills[dir::Up]);
		}
		
		if !spills[dir::Down].is_filled(false) {
			self.mask(origin, dir::Down, origin.with_y(15)).layer_zx_mut(15).combine(&spills[dir::Down]);
		}

		if !spills[dir::PlusX].is_filled(false) {
			self.mask(origin, dir::PlusX, origin.with_x(0)).layer_zy_mut(0).combine(&spills[dir::PlusX]);
		}
	
		if !spills[dir::MinusX].is_filled(false) {
			self.mask(origin, dir::MinusX, origin.with_x(15)).layer_zy_mut(15).combine(&spills[dir::MinusX]);
		}
	
		if !spills[dir::PlusZ].is_filled(false) {
			self.mask(origin, dir::PlusZ, origin.with_z(0)).layer_yx_mut(0).combine(&spills[dir::PlusZ]);
		}
	
		if !spills[dir::MinusZ].is_filled(false) {
			self.mask(origin, dir::MinusZ, origin.with_z(15)).layer_yx_mut(15).combine(&spills[dir::MinusZ]);
		}
	}

	fn mask<D>(&mut self, origin: ChunkPosition, dir: D, wrapped: ChunkPosition) -> Guard<NoPack<ChunkMask>>
		where ChunkPosition: Offset<D>, Directional<&'a SharedSector<NoPack<ChunkMask>>>: IndexMut<D, Output=&'a SharedSector<NoPack<ChunkMask>>>, D: Copy {
		
		match origin.offset(dir) {
			Some(internal) => self.spills.get_or_create(internal),
			None => self.neighbors[dir].get_or_create(wrapped)
		}
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

fn initial_sector(block_sector: &Sector<ChunkIndexed<Block>>, sky_light: &mut SharedSector<NoPack<ChunkNibbles>>, mut spills: SectorSpills) -> Box<[ColumnHeightMap]> {
	let mut heightmaps: Vec<ColumnHeightMap> = Vec::with_capacity(256);

	let lighting_info = lighting_info();
	let empty_lighting = ChunkNibbles::default();
	let mut queue = Queue::default();

	for (position, column) in block_sector.enumerate_columns() {
		let mut mask = LayerMask::default();
		let mut heightmap_builder = HeightMapBuilder::new();

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

			let old_spills = queue.reset_spills();

			let chunk_position = ChunkPosition::from_layer(y as u8, position);

			spills.spill_out(chunk_position, old_spills);
			sky_light.set(chunk_position, NoPack(light_data));
		}

		let heightmap = heightmap_builder.build();
		heightmaps.push(heightmap);
	}

	heightmaps.into_boxed_slice()
}

pub fn compute_skylight(world: &World<ChunkIndexed<Block>>) -> (SharedWorld<NoPack<ChunkNibbles>>, HashMap<(i32, i32), ColumnHeightMap>) {
	let mut sky_light = SharedWorld::<NoPack<ChunkNibbles>>::new();
	let mut incomplete = SharedWorld::<NoPack<ChunkMask>>::new();
	let mut heightmaps = HashMap::<(i32, i32), ColumnHeightMap>::new(); // TODO: Better vocs integration.

	let lighting_info = lighting_info();

	let empty_lighting = ChunkNibbles::default();
	let void_sector_above: SharedSector<NoPack<ChunkMask>> = SharedSector::new();
	let void_sector_below: SharedSector<NoPack<ChunkMask>> = SharedSector::new();

	let mut queue = Queue::default();

	println!("Performing initial sky lighting for region (0, 0)");
	let lighting_start = Instant::now();

	for sector_z in 0..2 {
		for sector_x in 0..2 {
			println!("Performing initial sky lighting for sector ({}, {})", sector_x, sector_z);

			let position = GlobalSectorPosition::new(sector_x, sector_z);

			let block_sector = match world.get_sector(position) {
				Some(sector) => sector,
				None => continue
			};

			let sky_light = sky_light.get_or_create_sector_mut(position);

			incomplete.get_or_create_sector_mut(position);
			incomplete.get_or_create_sector_mut(GlobalSectorPosition::new(sector_x + 1, sector_z));
			incomplete.get_or_create_sector_mut(GlobalSectorPosition::new(sector_x - 1, sector_z));
			incomplete.get_or_create_sector_mut(GlobalSectorPosition::new(sector_x, sector_z + 1));
			incomplete.get_or_create_sector_mut(GlobalSectorPosition::new(sector_x, sector_z - 1));

			let spill_neighbors = SplitDirectional {
				up: &void_sector_above,
				down: &void_sector_below,
				plus_x: incomplete.get_sector(GlobalSectorPosition::new(sector_x + 1, sector_z)).unwrap(),
				minus_x: incomplete.get_sector(GlobalSectorPosition::new(sector_x - 1, sector_z)).unwrap(),
				plus_z: incomplete.get_sector(GlobalSectorPosition::new(sector_x, sector_z + 1)).unwrap(),
				minus_z: incomplete.get_sector(GlobalSectorPosition::new(sector_x, sector_z - 1)).unwrap()
			};

			let spills = SectorSpills {
				spills: incomplete.get_sector(position).unwrap(),
				neighbors: Directional::combine(spill_neighbors)
			};

			let sector_heightmaps = initial_sector(block_sector, sky_light, spills);

			for (index, heightmap) in sector_heightmaps.into_vec().drain(..).enumerate() {
				let layer = LayerPosition::from_zx(index as u8);
				let column_position = GlobalColumnPosition::combine(position, layer);

				println!("{} {}", column_position.x(), column_position.z());
				heightmaps.insert((column_position.x(), column_position.z()), heightmap);
			}
		}
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(lighting_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Initial sky lighting done in {}us ({}us per column)", us, us / 1024);
	}

	println!("Completing sky lighting for region (0, 0)");
	let complete_lighting_start = ::std::time::Instant::now();

	while incomplete.sectors().len() > 0 {
		let incomplete_front = ::std::mem::replace(&mut incomplete, SharedWorld::new());

		for (sector_position, sector) in incomplete_front.into_sectors() {
			// TODO: println!("Completing sector @ {} - {} queued", sector_position, sector.count_sectors());

			println!("Completing sector @ {} - <unknown> queued", sector_position);

			let block_sector = match world.get_sector(sector_position) {
				Some(sector) => sector,
				None => continue, // No sense in lighting the void.
			};

			println!("(not skipped)");

			let light_sector = sky_light.get_or_create_sector_mut(sector_position);

			// TODO: while let Some((position, incomplete)) = sector.pop_first() {
			for position in ChunkPosition::enumerate() {
				let incomplete = match sector.remove(position) {
					Some(incomplete) => incomplete.0,
					None => continue
				};

				// use vocs::mask::Mask;
				// println!("Completing chunk: {} / {} queued blocks", position, incomplete.count_ones());

				let (blocks, palette) = block_sector[position].as_ref().unwrap().freeze();

				let mut opacity = BulkNibbles::new(palette.len());

				for (index, value) in palette.iter().enumerate() {
					opacity.set(
						index,
						value
							.and_then(|entry| lighting_info.get(&entry).map(|opacity| *opacity))
							.unwrap_or(u4::new(15)),
					);
				}

				let column_pos = GlobalColumnPosition::combine(sector_position, position.layer());
				let heightmap = heightmaps.get(&(column_pos.x(), column_pos.z())).unwrap();


				let chunk_heightmap = heightmap.slice(u4::new(position.y()));
				let sources = SkyLightSources::new(&chunk_heightmap);

				// TODO: cross-sector lighting

				let mut central = light_sector.get_or_create(position);
				let locks = SplitDirectional {
					up: position.offset(dir::Up).map(|position| light_sector[position].read()),
					down: position.offset(dir::Down).map(|position| light_sector[position].read()),
					plus_x: position
						.offset(dir::PlusX)
						.map(|position| light_sector[position].read()),
					minus_x: position
						.offset(dir::MinusX)
						.map(|position| light_sector[position].read()),
					plus_z: position
						.offset(dir::PlusZ)
						.map(|position| light_sector[position].read()),
					minus_z: position
						.offset(dir::MinusZ)
						.map(|position| light_sector[position].read()),
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

				// TODO: Queue handling
			}
		}
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(complete_lighting_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Sky lighting completion done in {}us ({}us per column)", us, us / 1024);
	}

	(sky_light, heightmaps)
}
