use vocs::position::GlobalColumnPosition;
use vocs::indexed::LayerIndexed;
use climate::ClimateSource;
use i73_noise::sample::Sample;
use {Biome, Lookup};

pub struct BiomeSource {
	climate: ClimateSource,
	lookup:  Lookup<Biome>
}

impl BiomeSource {
	pub fn new(climate: ClimateSource, lookup: Lookup<Biome>) -> Self {
		BiomeSource { climate, lookup }
	}

	pub fn layer(&self, chunk: GlobalColumnPosition) -> LayerIndexed<Biome> {
		let block = (
			(chunk.x() * 16) as f64,
			(chunk.z() * 16) as f64
		);

		self.lookup.climates_to_biomes(&self.climate.chunk(block))
	}
}