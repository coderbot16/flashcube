use crate::sources::LightSources;
use vocs::nibbles::{u4, BulkNibbles, ChunkNibbles};
use vocs::packed::ChunkPacked;
use vocs::position::ChunkPosition;
use vocs::view::SpillChunkMask;

#[derive(Debug)]
pub struct BlockLightSources<'c> {
	emission: BulkNibbles,
	chunk: &'c ChunkPacked,
}

impl<'c> BlockLightSources<'c> {
	pub fn new(chunk: &'c ChunkPacked) -> Self {
		BlockLightSources { emission: BulkNibbles::new(1 << chunk.bits()), chunk }
	}

	pub fn set_emission(&mut self, raw_index: usize, value: u4) {
		self.emission.set(raw_index, value)
	}
}

impl<'c> LightSources for BlockLightSources<'c> {
	fn emission(&self, position: ChunkPosition) -> u4 {
		self.emission.get(self.chunk.get(position) as usize)
	}

	fn initial(&self, _data: &mut ChunkNibbles, _mask: &mut SpillChunkMask) {
		unimplemented!()
	}
}
