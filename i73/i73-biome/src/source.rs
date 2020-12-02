use crate::climate::ClimateSource;
use crate::{Biome, Lookup};
use i73_noise::sample::Sample;
use vocs::indexed::IndexedLayer;
use vocs::position::GlobalColumnPosition;

pub struct BiomeSource {
	climate: ClimateSource,
	lookup: Lookup<Biome>,
}

impl BiomeSource {
	pub fn new(climate: ClimateSource, lookup: Lookup<Biome>) -> Self {
		BiomeSource { climate, lookup }
	}

	pub fn layer(&self, chunk: GlobalColumnPosition) -> IndexedLayer<Biome> {
		let block = ((chunk.x() * 16) as f64, (chunk.z() * 16) as f64);

		self.lookup.climates_to_biomes(&self.climate.chunk(block))
	}
}
