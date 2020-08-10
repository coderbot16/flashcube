use crate::colorizer::colorize_grass;
use crate::renderer::{self, duration_us, Renderer};
use i73_base::{math, Layer, Pass};
use i73_base::block::{self, Block};
use i73_biome::climate::{Climate, ClimateSource};
use crate::Rgb;
use i73_noise::sample::Sample;
use i73_terrain::overworld::ocean::{OceanBlocks, OceanPass};
use i73_terrain::overworld::paint::PaintPass;
use i73_terrain::overworld::shape::ShapePass;
use i73_terrain::overworld_173;
use i73_terrain::overworld_173::Settings;
use image::{GenericImage, RgbImage, SubImage};
use std::fmt::{self, Display};
use std::ops::AddAssign;
use vocs::indexed::ChunkIndexed;
use vocs::position::{ColumnPosition, GlobalColumnPosition, GlobalSectorPosition, LayerPosition};
use vocs::view::ColumnMut;

// Block types
const AIR: Block = block::AIR;
const STONE: Block = block::STONE;
const GRASS: Block = block::GRASS;
const DIRT: Block = block::DIRT;
const BEDROCK: Block = block::BEDROCK;
const OCEAN: Block = block::STILL_WATER;
const SAND: Block = block::SAND;
const GRAVEL: Block = block::GRAVEL;
const ICE: Block = block::ICE;

type OverworldPasses = (ClimateSource, ShapePass, PaintPass);

#[derive(Default)]
pub struct TimeMetrics {
	pub total: u64,
	pub climates: u64,
	pub shape: u64,
	pub paint: u64,
	pub ocean: u64,
}

impl TimeMetrics {
	pub fn climates_percentage(&self) -> f64 {
		(self.climates as f64 / self.total as f64) * 100.0
	}

	pub fn shape_percentage(&self) -> f64 {
		(self.shape as f64 / self.total as f64) * 100.0
	}

	pub fn paint_percentage(&self) -> f64 {
		(self.paint as f64 / self.total as f64) * 100.0
	}

	pub fn ocean_percentage(&self) -> f64 {
		(self.ocean as f64 / self.total as f64) * 100.0
	}
}

impl Display for TimeMetrics {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{:8.3}ms, {:5.3}ms/column | {:5.2}% / {:5.2}% / {:5.2}% / {:5.2}%",
			(self.total as f64) / 1000.0,
			(self.total / 256) as f64 / 1000.0,
			self.climates_percentage(),
			self.shape_percentage(),
			self.paint_percentage(),
			self.ocean_percentage()
		)
	}
}

#[derive(Default)]
pub struct TotalMetrics {
	pub climates: u64,
	pub shape: u64,
	pub paint: u64,
	pub ocean: u64,
	pub total: u64,
	pub thread_count: u32,
}

impl TotalMetrics {
	pub fn climates_percentage(&self) -> f64 {
		(self.climates as f64 / self.total as f64) * 100.0
	}

	pub fn shape_percentage(&self) -> f64 {
		(self.shape as f64 / self.total as f64) * 100.0
	}

	pub fn paint_percentage(&self) -> f64 {
		(self.paint as f64 / self.total as f64) * 100.0
	}

	pub fn ocean_percentage(&self) -> f64 {
		(self.ocean as f64 / self.total as f64) * 100.0
	}
}

impl renderer::TotalMetrics for TotalMetrics {
	fn set_thread_count(&mut self, threads: u32) {
		self.thread_count = threads;
	}

	fn set_duration_us(&mut self, time: u64) {
		self.total = time;
	}
}

impl AddAssign<TimeMetrics> for TotalMetrics {
	fn add_assign(&mut self, other: TimeMetrics) {
		self.climates += other.climates;
		self.shape += other.shape;
		self.paint += other.paint;
		self.ocean += other.ocean;
	}
}

impl Display for TotalMetrics {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"{:5.2}% / {:5.2}% / {:5.2}% / {:5.2}%",
			self.climates_percentage() / (self.thread_count as f64),
			self.shape_percentage() / (self.thread_count as f64),
			self.paint_percentage() / (self.thread_count as f64),
			self.ocean_percentage() / (self.thread_count as f64)
		)
	}
}

pub fn create_renderer(seed: u64) -> FullRenderer {
	let settings = Settings::default();

	let ocean = OceanPass {
		blocks: OceanBlocks {
			ocean: block::STILL_WATER,
			air: block::AIR,
			ice: block::ICE,
		},
		sea_top: (settings.sea_coord + 1) as usize,
	};

	let biome_lookup = frontend::generate_biome_lookup();
	let passes = overworld_173::passes(seed, settings, biome_lookup);

	FullRenderer { passes, ocean }
}

pub struct FullRenderer {
	passes: OverworldPasses,
	ocean: OceanPass,
}

impl Renderer for FullRenderer {
	type SectorMetrics = TimeMetrics;
	type TotalMetrics = TotalMetrics;

	fn process_sector(&self, sector_position: GlobalSectorPosition) -> (RgbImage, TimeMetrics) {
		let (ref climates, ref shape, ref paint) = &self.passes;

		let gen_start = ::std::time::Instant::now();
		let mut map = RgbImage::new(256, 256);

		let mut metrics = TimeMetrics::default();

		for layer_position in LayerPosition::enumerate() {
			let column_position = GlobalColumnPosition::combine(sector_position, layer_position);

			let mut column_chunks = [
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
				ChunkIndexed::<Block>::new(4, block::AIR),
			];

			let mut column: ColumnMut<Block> = ColumnMut::from_array(&mut column_chunks);

			let pass_start = ::std::time::Instant::now();
			let climates = climates
				.chunk(((column_position.x() * 16) as f64, (column_position.z() * 16) as f64));
			metrics.climates += duration_us(&pass_start);

			//metrics("initial", &column, x, z);
			shape.apply(&mut column, &climates, column_position);
			metrics.shape += duration_us(&pass_start);

			//metrics("shape", &column, x, z);
			paint.apply(&mut column, &climates, column_position);
			metrics.paint += duration_us(&pass_start);

			//metrics("paint", &column, x, z);
			self.ocean.apply(&mut column, &climates, column_position);
			metrics.ocean += duration_us(&pass_start);

			//metrics("ocean", &column, x, z);

			let target = SubImage::new(
				&mut map,
				layer_position.x() as u32 * 16,
				layer_position.z() as u32 * 16,
				16,
				16,
			);

			self.render_column(&column, target, &climates);
		}

		metrics.total = duration_us(&gen_start);
		metrics.ocean -= metrics.paint;
		metrics.paint -= metrics.shape;
		metrics.shape -= metrics.climates;

		(map, metrics)
	}
}

impl FullRenderer {
	fn render_column(
		&self, column: &ColumnMut<Block>, mut target: SubImage<&mut RgbImage>,
		climates: &Layer<Climate>,
	) {
		for layer_position in LayerPosition::enumerate() {
			let mut height = 0;
			let mut ocean_height = 0;
			let mut ice = false;

			for cy in (0..128).rev() {
				let column_position = ColumnPosition::from_layer(cy, layer_position);
				let block = *column.get(column_position);

				let ocean = block == OCEAN || block == ICE;
				let solid = block != AIR;

				if ocean_height == 0 && ocean {
					ocean_height = cy;
					ice = block == ICE;
					continue;
				}

				if solid && !ocean {
					height = cy;
					break;
				}
			}

			let position = ColumnPosition::from_layer(height, layer_position);
			let top = *column.get(position);

			let climate = climates.get(layer_position);
			let mut no_shade = false;

			let color = match top {
				AIR => Rgb::gray(255),
				STONE => Rgb::gray(127),
				GRASS => colorize_grass(climate),
				DIRT => Rgb { red: 255, green: 196, blue: 127 },
				BEDROCK => Rgb::gray(0),
				SAND => Rgb { red: 255, green: 240, blue: 127 },
				GRAVEL => Rgb::gray(196),
				_ => {
					println!("warning: unknown block: {:?}", top);
					no_shade = true;

					Rgb { red: 255, green: 0, blue: 255 }
				}
			};

			let shaded_color = if no_shade {
				color
			} else if ocean_height != 0 {
				let depth = ocean_height - height;
				let shade = math::clamp(depth as f64 / 32.0, 0.0, 1.0);
				let shade = 1.0 - (1.0 - shade).powi(2);

				if !ice {
					Rgb {
						red: (color.red as f64 * (1.0 - shade) * 0.5) as u8,
						green: (color.green as f64 * (1.0 - shade) * 0.5) as u8,
						blue: math::lerp(color.blue as f64, 255.0, shade) as u8,
					}
				} else {
					Rgb {
						red: math::lerp(color.green as f64 * 0.5 + 63.0, 63.0, shade) as u8,
						green: math::lerp(color.green as f64 * 0.5 + 63.0, 63.0, shade) as u8,
						blue: math::lerp(color.blue as f64, 255.0, shade) as u8,
					}
				}
			} else {
				let shade = math::clamp(((height as f64) / 127.0) * 0.6 + 0.4, 0.0, 1.0);

				let (color, shade) = if climate.freezing() {
					(Rgb::gray(255), 1.0 - (1.0 - shade).powi(2))
				} else {
					(color, shade)
				};

				Rgb {
					red: (color.red as f64 * shade) as u8,
					green: (color.green as f64 * shade) as u8,
					blue: (color.blue as f64 * shade) as u8,
				}
			};

			target.put_pixel(layer_position.x() as u32, layer_position.z() as u32, shaded_color.into());
		}
	}
}
