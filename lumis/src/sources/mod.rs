use vocs::nibbles::{u4, ChunkNibbles};
use vocs::position::ChunkPosition;
use vocs::view::SpillChunkMask;

mod block;
mod sky;

pub use block::BlockLightSources;
pub use sky::SkyLightSources;

pub trait LightSources {
	fn emission(&self, position: ChunkPosition) -> u4;
	fn initial(&self, data: &mut ChunkNibbles, mask: &mut SpillChunkMask);
}
