use crate::renderer::{duration_us, Renderer, BasicTimeMetrics, BasicTotalMetrics};
use i73_biome::climate::{ClimateSource, ClimateSettings, Climate};
use i73_noise::sample::Sample;
use image::{Rgb, RgbImage};
use vocs::position::{GlobalSectorPosition, LayerPosition, GlobalColumnPosition};

pub trait Mapper: Send + Sync {
	fn map(&self, climate: Climate) -> Rgb<u8>;
}

impl<F> Mapper for F where F: Send + Sync + Fn(Climate) -> Rgb<u8> {
	fn map(&self, climate: Climate) -> Rgb<u8> {
		self(climate)
	}
}

/// Renders climates to colors using a user-provided mapping function.
pub struct ClimateRenderer<F>(ClimateSource, F) where F: Mapper;

impl<F> ClimateRenderer<F> where F: Mapper {
	pub fn new(seed: u64, f: F) -> Self {
		let climates = ClimateSource::new(seed, ClimateSettings::default());

		ClimateRenderer(climates, f)
	}
}

impl<F> Renderer for ClimateRenderer<F> where F: Mapper {
	type SectorMetrics = BasicTimeMetrics;
	type TotalMetrics = BasicTotalMetrics;

	fn process_sector(&self, sector_position: GlobalSectorPosition) -> (RgbImage, BasicTimeMetrics) {
		let gen_start = ::std::time::Instant::now();
		let mut map = RgbImage::new(256, 256);

		let mut metrics = BasicTimeMetrics::default();

		for layer_position in LayerPosition::enumerate() {
			let column_position = GlobalColumnPosition::combine(sector_position, layer_position);

			let climates = self.0.chunk((
				(column_position.x() * 16) as f64,
				(column_position.z() * 16) as f64
			));

			let base = (
				layer_position.x() as u32 * 16,
				layer_position.z() as u32 * 16
			);

			for pixel in LayerPosition::enumerate() {
				map.put_pixel(
					base.0 + pixel.x() as u32,
					base.1 + pixel.z() as u32,
					self.1.map(climates.get(pixel))
				);
			}
		}

		metrics.total = duration_us(&gen_start);

		(map, metrics)
	}
}