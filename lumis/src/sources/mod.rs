use vocs::nibbles::{u4, ChunkNibbles};
use vocs::position::ChunkPosition;
use vocs::view::SpillChunkMask;

pub mod block;
pub mod sky;

pub trait LightSources {
	fn emission(&self, position: ChunkPosition) -> u4;
	fn initial(&self, data: &mut ChunkNibbles, mask: &mut SpillChunkMask);
}
