use crate::sources::LightSources;
use crate::PackedNibbleCube;
use vocs::indexed::{Target, IndexedCube};
use vocs::nibbles::{u4, NibbleArray, NibbleCube};
use vocs::packed::PackedCube;
use vocs::position::{dir, CubePosition, GlobalSectorPosition};
use vocs::view::{MaskOffset, SpillBitCube};
use vocs::world::world::World;
use vocs::world::sector::Sector;
use std::marker::PhantomData;

pub trait EmissionPalette<B: Target>: Sync {
	fn emission(&self, block: &B) -> u4;
}

impl<B, T> EmissionPalette<B> for T where B: Target, T: Fn(&B) -> u4 + Sync {
	fn emission(&self, block: &B) -> u4 {
		self(block)
	}
}

#[derive(Debug)]
pub struct BlockLightSources<B: Target, E: EmissionPalette<B>> {
	emission_array: NibbleArray,
	phantom_block: PhantomData<B>,
	phantom_emission: PhantomData<E>
}

impl<B: Target, E: EmissionPalette<B>> BlockLightSources<B, E> {
	pub fn new(chunk: &PackedCube) -> Self {
		BlockLightSources {
			emission_array: NibbleArray::new(1 << chunk.bits()),
			phantom_block: PhantomData,
			phantom_emission: PhantomData
		}
	}

	pub fn set_emission(&mut self, raw_index: usize, value: u4) {
		self.emission_array.set(raw_index, value)
	}
}

impl<B: Target + Sync, E: EmissionPalette<B> + Sync> LightSources for BlockLightSources<B, E> {
	type SectorSources = Sector<IndexedCube<B>>;
	type WorldSources = World<IndexedCube<B>>;
	type EmissionPalette = E;

	fn sector_sources(world_sources: &Self::WorldSources, position: GlobalSectorPosition) -> &Self::SectorSources {
		world_sources.get_sector(position).unwrap()
	}

	fn chunk_sources(sector_sources: &Self::SectorSources, emission_palette: &Self::EmissionPalette, position: CubePosition) -> Self {
		let chunk = &sector_sources[position];
		let (blocks, palette) = chunk.as_ref().unwrap().freeze();

		let mut sources: BlockLightSources<B, E> = BlockLightSources::new(blocks);

		for (index, entry) in palette.iter().enumerate() {
			let emission = entry.as_ref().map(|entry| emission_palette.emission(entry));

			sources.set_emission(index, emission.unwrap_or(u4::ZERO));
		}

		sources
	}

	fn emission(&self, blocks: &PackedCube, position: CubePosition) -> u4 {
		self.emission_array.get(blocks.get(position) as usize)
	}

	fn initial(&self, blocks: &PackedCube, enqueued: &mut SpillBitCube) -> PackedNibbleCube {
		// Optimization: Don't bother performing initial block lighting on chunks that don't have
		// any block light emitters to begin with

		let mut has_emitters = false;

		for emission in self.emission_array.iter() {
			if emission != u4::ZERO {
				has_emitters = true;
				break;
			}
		}

		if !has_emitters {
			return PackedNibbleCube::EntirelyDark
		}

		let mut data = NibbleCube::default();
		let mut set_any = false;

		// If the chunk does have emitters, figure out where they are and place the correct initial
		// light sources
		for position in CubePosition::enumerate() {
			let emission = self.emission(blocks, position);

			// Nothing to do at this position
			if emission == u4::ZERO {
				continue;
			}

			// We get to assume that data is all zeroes
			data.set_uncleared(position, emission);
			set_any = true;
			
			if emission == u4::ONE {
				// A light level of 1 does not result in neighbor propagation
				continue;
			}

			// Enqueue all neighbors
			enqueued.set_offset_true(position, dir::MinusX);
			enqueued.set_offset_true(position, dir::MinusZ);
			enqueued.set_offset_true(position, dir::PlusX);
			enqueued.set_offset_true(position, dir::PlusZ);
			enqueued.set_offset_true(position, dir::Down);
			enqueued.set_offset_true(position, dir::Up);
		}

		if set_any {
			PackedNibbleCube::Unpacked(data)
		} else {
			// This should be a rare occurrence, but it's possible that there are stale palette
			// entries in the chunk
			PackedNibbleCube::EntirelyDark
		}
	}
}
