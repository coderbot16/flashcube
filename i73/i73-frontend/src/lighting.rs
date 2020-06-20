use i73_base::Block;

use rs25::dynamics::light::{HeightMapBuilder, Lighting, SkyLightSources};
use rs25::dynamics::queue::Queue;

use std::collections::HashMap;
use std::time::Instant;

use vocs::component::LayerStorage;
use vocs::indexed::ChunkIndexed;
use vocs::mask::ChunkMask;
use vocs::mask::LayerMask;
use vocs::nibbles::{u4, BulkNibbles, ChunkNibbles};
use vocs::position::{dir, Offset, GlobalChunkPosition, GlobalColumnPosition};
use vocs::view::{Directional, SplitDirectional};
use vocs::world::shared::{NoPack, SharedWorld};
use vocs::world::world::World;

fn spill_out(
	chunk_position: GlobalChunkPosition, incomplete: &mut World<ChunkMask>,
	old_spills: Directional<LayerMask>,
) {
	if let Some(up) = chunk_position.plus_y() {
		if !old_spills[dir::Up].is_filled(false) {
			incomplete.get_or_create_mut(up).layer_zx_mut(0).combine(&old_spills[dir::Up]);
		}
	}

	if let Some(down) = chunk_position.minus_y() {
		if !old_spills[dir::Down].is_filled(false) {
			incomplete.get_or_create_mut(down).layer_zx_mut(15).combine(&old_spills[dir::Down]);
		}
	}

	if let Some(plus_x) = chunk_position.plus_x() {
		if !old_spills[dir::PlusX].is_filled(false) {
			incomplete
				.get_or_create_mut(plus_x)
				.layer_zy_mut(0)
				.combine(&old_spills[dir::PlusX]);
		}
	}

	if let Some(minus_x) = chunk_position.minus_x() {
		if !old_spills[dir::MinusX].is_filled(false) {
			incomplete
				.get_or_create_mut(minus_x)
				.layer_zy_mut(15)
				.combine(&old_spills[dir::MinusX]);
		}
	}

	if let Some(plus_z) = chunk_position.plus_z() {
		if !old_spills[dir::PlusZ].is_filled(false) {
			incomplete
				.get_or_create_mut(plus_z)
				.layer_yx_mut(0)
				.combine(&old_spills[dir::PlusZ]);
		}
	}

	if let Some(minus_z) = chunk_position.minus_z() {
		if !old_spills[dir::MinusZ].is_filled(false) {
			incomplete
				.get_or_create_mut(minus_z)
				.layer_yx_mut(15)
				.combine(&old_spills[dir::MinusZ]);
		}
	}
}

pub fn compute_skylight(world: &World<ChunkIndexed<Block>>) -> (SharedWorld<NoPack<ChunkNibbles>>, HashMap<(i32, i32), Vec<u32>>) {
	let mut sky_light = SharedWorld::<NoPack<ChunkNibbles>>::new();
	let mut incomplete = World::<ChunkMask>::new();
	let mut heightmaps = HashMap::<(i32, i32), Vec<u32>>::new(); // TODO: Better vocs integration.

	let mut lighting_info = HashMap::new()/*SparseStorage::<u4>::with_default(u4::new(15))*/;
	lighting_info.insert(Block::air(), u4::new(0));
	lighting_info.insert(Block::from_anvil_id(8 * 16), u4::new(2));
	lighting_info.insert(Block::from_anvil_id(9 * 16), u4::new(2));
	lighting_info.insert(Block::from_anvil_id(18 * 16), u4::new(1));

	let empty_lighting = ChunkNibbles::default();

	let mut queue = Queue::default();

	println!("Performing initial sky lighting for region (0, 0)");
	let lighting_start = Instant::now();

	for x in 0..32 {
		println!("{}", x);
		for z in 0..32 {
			let column_position = GlobalColumnPosition::new(x, z);

			let mut mask = LayerMask::default();
			let mut heightmap = HeightMapBuilder::new();
			let mut heightmap_sections = [
				None, None, None, None, None, None, None, None, None, None, None, None, None, None,
				None, None,
			];

			for y in (0..16).rev() {
				let chunk_position = GlobalChunkPosition::from_column(column_position, y);

				let (blocks, palette) = world.get(chunk_position).unwrap().freeze();

				let mut opacity = BulkNibbles::new(palette.len());

				for (index, value) in palette.iter().enumerate() {
					opacity.set(
						index,
						value
							.and_then(|entry| lighting_info.get(&entry).map(|opacity| *opacity))
							.unwrap_or(u4::new(15)),
					);
				}

				let sources = SkyLightSources::build(blocks, &opacity, mask);

				let mut light_data = ChunkNibbles::default();
				let neighbors = Directional::combine(SplitDirectional {
					minus_x: &empty_lighting,
					plus_x: &empty_lighting,
					minus_z: &empty_lighting,
					plus_z: &empty_lighting,
					down: &empty_lighting,
					up: &empty_lighting,
				});

				let sources = {
					let mut light = Lighting::new(&mut light_data, neighbors, sources, opacity);

					light.initial(blocks, &mut queue);
					light.finish(blocks, &mut queue);

					light.decompose().1
				};

				heightmap_sections[y as usize] = Some(sources.clone());
				mask = heightmap.add(sources);

				let old_spills = queue.reset_spills();

				spill_out(chunk_position, &mut incomplete, old_spills);

				sky_light.set(chunk_position, NoPack(light_data));
			}

			let heightmap = heightmap.build();

			/*for (index, part) in heightmap_sections.iter().enumerate() {
				let part = part.as_ref().unwrap().clone();

				assert_eq!(SkyLightSources::slice(&heightmap, index as u8), part);
			}*/

			heightmaps.insert((x, z), heightmap.into_vec());
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
		let incomplete_front = ::std::mem::replace(&mut incomplete, World::new());

		for (sector_position, mut sector) in incomplete_front.into_sectors() {
			println!("Completing sector @ {} - {} queued", sector_position, sector.count_sectors());

			let block_sector = match world.get_sector(sector_position) {
				Some(sector) => sector,
				None => continue, // No sense in lighting the void.
			};

			println!("(not skipped)");

			let light_sector = sky_light.get_or_create_sector_mut(sector_position);

			while let Some((position, incomplete)) = sector.pop_first() {
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

				let sources = SkyLightSources::slice(&heightmap, position.y());

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
				light.finish(blocks, &mut queue);

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
