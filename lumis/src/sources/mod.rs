use vocs::nibbles::{u4, NibbleCube};
use vocs::position::{CubePosition, GlobalSectorPosition};
use vocs::view::SpillBitCube;

mod block;
mod sky;

pub use block::BlockLightSources;
pub use sky::SkyLightSources;

pub trait LightSources {
	type SectorSources: Sync;
	type WorldSources: Sync;

	fn sector_sources(world_sources: &Self::WorldSources, position: GlobalSectorPosition) -> &Self::SectorSources;
	fn chunk_sources(sector_sources: &Self::SectorSources, position: CubePosition) -> Self;

	fn emission(&self, position: CubePosition) -> u4;
	fn initial(&self, data: &mut NibbleCube, mask: &mut SpillBitCube);
}

pub trait RefSync {
}

impl<'a, T: 'a> RefSync for T where &'a T: Sync {
}
