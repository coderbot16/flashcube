use vocs::nibbles::{u4, ChunkNibbles};
use vocs::position::CubePosition;
use vocs::view::SpillChunkMask;

mod block;
mod sky;

pub use block::BlockLightSources;
pub use sky::SkyLightSources;

pub trait LightSources {
	fn emission(&self, position: CubePosition) -> u4;
	fn initial(&self, data: &mut ChunkNibbles, mask: &mut SpillChunkMask);
}
