use cgmath::{Point2, Vector2};
use vocs::position::{LayerPosition, GlobalColumnPosition};
use vocs::indexed::LayerIndexed;
use {Biome, Lookup};
use climate::{Climate, ClimateSource};
use i73_noise::sample::Sample;

pub struct BiomeSource {
	climate: ClimateSource,
	lookup:  Lookup
}

impl BiomeSource {
	pub fn new(climate: ClimateSource, lookup: Lookup) -> Self {
		BiomeSource { climate, lookup }
	}
	
	pub fn layer(&self, chunk: GlobalColumnPosition) -> LayerIndexed<Biome> {
		let block = Point2::new (
			(chunk.x() * 16) as f64,
			(chunk.z() * 16) as f64
		);

		// TODO: Avoid the default lookup and clone.
		let mut layer = LayerIndexed::new(2, self.lookup.lookup(Climate::new(1.0, 1.0)).clone());

		for position in LayerPosition::enumerate() {
			let climate = self.climate.sample(block + Vector2::new(position.x() as f64, position.z() as f64));
			let biome = self.lookup.lookup(climate);

			layer.set_immediate(position, biome);
		}
		
		layer
	}
}