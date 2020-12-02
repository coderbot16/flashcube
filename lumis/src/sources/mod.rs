use vocs::nibbles::{u4, NibbleCube};
use vocs::position::CubePosition;
use vocs::view::SpillBitCube;

mod block;
mod sky;

pub use block::BlockLightSources;
pub use sky::SkyLightSources;

pub trait LightSources {
	fn emission(&self, position: CubePosition) -> u4;
	fn initial(&self, data: &mut NibbleCube, mask: &mut SpillBitCube);
}
