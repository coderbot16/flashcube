use vocs::nibbles::{u4, NibbleCube};
use vocs::packed::PackedCube;
use vocs::position::{CubePosition, GlobalSectorPosition};
use vocs::view::SpillBitCube;

mod block;
mod sky;

pub use block::{BlockLightSources, EmissionPalette};
pub use sky::SkyLightSources;

pub trait LightSources {
	type SectorSources: Sync;
	type WorldSources: Sync;
	type EmissionPalette: Sync;

	fn sector_sources(world_sources: &Self::WorldSources, position: GlobalSectorPosition) -> &Self::SectorSources;
	fn chunk_sources(sector_sources: &Self::SectorSources, emission_palette: &Self::EmissionPalette, position: CubePosition) -> Self;

	fn emission(&self, blocks: &PackedCube, position: CubePosition) -> u4;
	fn initial(&self, blocks: &PackedCube, data: &mut NibbleCube, mask: &mut SpillBitCube);
}

pub trait RefSync {
}

impl<'a, T: 'a> RefSync for T where &'a T: Sync {
}
