use bit_vec::BitVec;

use i73_base::Block;

use lumis::heightmap::{ChunkHeightMap, ColumnHeightMap, HeightMapBuilder};
use lumis::light::Lighting;
use lumis::sources::SkyLightSources;
use lumis::queue::{DirectionSpills, Queue};

use std::collections::HashMap;
use std::time::Instant;

use vocs::component::LayerStorage;
use vocs::indexed::ChunkIndexed;
use vocs::mask::ChunkMask;
use vocs::mask::LayerMask;
use vocs::nibbles::{u4, BulkNibbles, ChunkNibbles};
use vocs::position::{dir, Offset, ChunkPosition, GlobalColumnPosition, GlobalSectorPosition, LayerPosition};
use vocs::view::{Directional, SplitDirectional, SpillChunkMask, MaskOffset};
use vocs::world::sector::Sector;
use vocs::world::shared::{NoPack, SharedSector, SharedWorld};
use vocs::world::world::World;

struct SectorSpills {
	layers: SplitDirectional<DirectionSpills>,
	queued: SpillChunkMask
}

impl SectorSpills {
	fn new() -> Self {
		SectorSpills {
			layers: SplitDirectional {
				plus_x:  DirectionSpills::new(),
				minus_x: DirectionSpills::new(),
				up:      DirectionSpills::new(),
				down:    DirectionSpills::new(),
				plus_z:  DirectionSpills::new(),
				minus_z: DirectionSpills::new()
			},
			queued: SpillChunkMask::default()
		}
	}

	fn spill_out(&mut self, origin: ChunkPosition, spills: SplitDirectional<LayerMask>) {
		if !spills.plus_x.is_filled(false) {
			self.queued.set_offset_true(origin, dir::PlusX);
			self.layers.plus_x.set(origin, spills.plus_x);
		}

		if !spills.minus_x.is_filled(false) {
			self.queued.set_offset_true(origin, dir::MinusX);
			self.layers.minus_x.set(origin, spills.minus_x);
		}

		if !spills.up.is_filled(false) {
			self.queued.set_offset_true(origin, dir::Up);
			self.layers.up.set(origin, spills.up);
		}

		if !spills.down.is_filled(false) {
			self.queued.set_offset_true(origin, dir::Down);
			self.layers.down.set(origin, spills.down);
		}

		if !spills.plus_z.is_filled(false) {
			self.queued.set_offset_true(origin, dir::PlusZ);
			self.layers.plus_z.set(origin, spills.plus_z);
		}

		if !spills.minus_z.is_filled(false) {
			self.queued.set_offset_true(origin, dir::MinusZ);
			self.layers.minus_z.set(origin, spills.minus_z);
		}
	}

	fn reset(&mut self) {
		assert!(self.queued.primary.empty());

		self.layers.plus_x.reset();
		self.layers.minus_x.reset();
		self.layers.up.reset();
		self.layers.down.reset();
		self.layers.plus_z.reset();
		self.layers.minus_z.reset();
	}

	fn empty(&self) -> bool {
		self.queued.primary.empty()
	}

	fn pop(&mut self) -> Option<(ChunkPosition, ChunkMask)> {
		let position = match self.queued.primary.pop_first() {
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
		incoming.minus_z.map(|layer| mask.layer_yx_mut(0).combine(&layer));

		Some((position, mask))
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

fn initial_sector(block_sector: &Sector<ChunkIndexed<Block>>, sky_light: &SharedSector<NoPack<ChunkNibbles>>) -> (SectorSpills, Box<[ColumnHeightMap]>) {
	let mut heightmaps: Vec<ColumnHeightMap> = Vec::with_capacity(256);

	let lighting_info = lighting_info();
	let empty_lighting = ChunkNibbles::default();
	let mut queue = Queue::default();
	let mut spills = SectorSpills::new();

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

			let old_spills = queue.reset_spills().split();

			let chunk_position = ChunkPosition::from_layer(y as u8, position);

			spills.spill_out(chunk_position, old_spills);
			sky_light.set(chunk_position, NoPack(light_data));
		}

		let heightmap = heightmap_builder.build();
		heightmaps.push(heightmap);
	}

	(spills, heightmaps.into_boxed_slice())
}

fn full_sector(block_sector: &Sector<ChunkIndexed<Block>>, sky_light: &SharedSector<NoPack<ChunkNibbles>>) -> Box<[ColumnHeightMap]> {
	let initial_start = Instant::now();

	let (mut spills, heightmaps) = initial_sector(block_sector, sky_light);
	
	let mut new_spills = SectorSpills::new();

	let lighting_info = lighting_info();
	let empty_lighting = ChunkNibbles::default();
	let mut queue = Queue::default();

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

			let heightmap = &heightmaps[position.layer().zx() as usize];

			let chunk_heightmap = heightmap.slice(u4::new(position.y()));
			let sources = SkyLightSources::new(&chunk_heightmap);

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

pub fn compute_skylight(world: &World<ChunkIndexed<Block>>) -> (SharedWorld<NoPack<ChunkNibbles>>, HashMap<(i32, i32), ColumnHeightMap>) {
	let mut sky_light = SharedWorld::<NoPack<ChunkNibbles>>::new();
	let mut heightmaps = HashMap::<(i32, i32), ColumnHeightMap>::new(); // TODO: Better vocs integration.

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

			for (index, heightmap) in sector_heightmaps.into_vec().drain(..).enumerate() {
				let layer = LayerPosition::from_zx(index as u8);
				let column_position = GlobalColumnPosition::combine(position, layer);

				heightmaps.insert((column_position.x(), column_position.z()), heightmap);
			}
		}
	}

	(sky_light, heightmaps)
}
