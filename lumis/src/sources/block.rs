use crate::sources::LightSources;
use vocs::nibbles::{u4, NibbleArray, NibbleCube};
use vocs::packed::PackedCube;
use vocs::position::{dir, CubePosition};
use vocs::view::{MaskOffset, SpillBitCube};

#[derive(Debug)]
pub struct BlockLightSources<'c> {
	emission: NibbleArray,
	chunk: &'c PackedCube,
}

impl<'c> BlockLightSources<'c> {
	pub fn new(chunk: &'c PackedCube) -> Self {
		BlockLightSources {
			emission: NibbleArray::new(1 << chunk.bits()),
			chunk
		}
	}

	pub fn set_emission(&mut self, raw_index: usize, value: u4) {
		self.emission.set(raw_index, value)
	}
}

impl<'c> LightSources for BlockLightSources<'c> {
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
