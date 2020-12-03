use crate::sources::LightSources;
use vocs::indexed::{Target, IndexedCube};
use vocs::nibbles::{u4, NibbleArray, NibbleCube};
use vocs::packed::PackedCube;
use vocs::position::{dir, CubePosition, GlobalSectorPosition};
use vocs::view::{MaskOffset, SpillBitCube};
use vocs::world::world::World;
use vocs::world::sector::Sector;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct BlockLightSources<'c, B: Target> {
	emission: NibbleArray,
	chunk: &'c PackedCube,
	phantom: PhantomData<B>
}

impl<'c, B: Target> BlockLightSources<'c, B> {
	pub fn new(chunk: &'c PackedCube) -> Self {
		BlockLightSources {
			emission: NibbleArray::new(1 << chunk.bits()),
			chunk,
			phantom: PhantomData
		}
	}

	pub fn set_emission(&mut self, raw_index: usize, value: u4) {
		self.emission.set(raw_index, value)
	}
}

impl<'c, B: Target + Sync> LightSources for BlockLightSources<'c, B> {
	type SectorSources = Sector<IndexedCube<B>>;
	type WorldSources = World<IndexedCube<B>>;

	fn sector_sources(world_sources: &Self::WorldSources, position: GlobalSectorPosition) -> &Self::SectorSources {
		todo!()
	}

	fn chunk_sources(sector_sources: &Self::SectorSources, position: CubePosition) -> Self {
		todo!()
	}

	fn emission(&self, position: CubePosition) -> u4 {
		self.emission.get(self.chunk.get(position) as usize)
	}

	fn initial(&self, data: &mut NibbleCube, enqueued: &mut SpillBitCube) {
		for position in CubePosition::enumerate() {
			let emission = self.emission(position);

			// Nothing to do at this position
			if emission == u4::ZERO {
				continue;
			}

			// We get to assume that data is all zeroes
			data.set_uncleared(position, emission);
			
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
	}
}
