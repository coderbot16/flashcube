use crate::sources::LightSources;
use vocs::nibbles::{u4, BulkNibbles, NibbleCube};
use vocs::packed::PackedCube;
use vocs::position::CubePosition;
use vocs::view::SpillBitCube;

#[derive(Debug)]
pub struct BlockLightSources<'c> {
	emission: BulkNibbles,
	chunk: &'c PackedCube,
}

impl<'c> BlockLightSources<'c> {
	pub fn new(chunk: &'c PackedCube) -> Self {
		BlockLightSources { emission: BulkNibbles::new(1 << chunk.bits()), chunk }
	}

	pub fn set_emission(&mut self, raw_index: usize, value: u4) {
		self.emission.set(raw_index, value)
	}
}

impl<'c> LightSources for BlockLightSources<'c> {
	fn emission(&self, position: CubePosition) -> u4 {
		self.emission.get(self.chunk.get(position) as usize)
	}

	fn initial(&self, _data: &mut NibbleCube, _mask: &mut SpillBitCube) {
		unimplemented!()
	}
}
