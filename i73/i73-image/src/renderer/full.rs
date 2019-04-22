use std::fmt::{self, Display};
use std::ops::AddAssign;
use i73_biome::climate::{Climate, ClimateSource};
use i73_noise::sample::Sample;
use i73_terrain::overworld::ocean::{OceanPass, OceanBlocks};
use i73_terrain::overworld_173;
use i73_biome::Lookup;
use i73_base::{Block, Pass, Layer, math};
use vocs::indexed::ChunkIndexed;
use vocs::view::ColumnMut;
use vocs::position::{GlobalColumnPosition, ColumnPosition, LayerPosition, GlobalSectorPosition};
use std::collections::HashMap;
use i73_terrain::overworld_173::Settings;
use frontend::config::biomes::{BiomesConfig, RectConfig, BiomeConfig, SurfaceConfig, FollowupConfig};
use colorizer::colorize_grass;
use i73_terrain::overworld::shape::ShapePass;
use i73_terrain::overworld::paint::PaintPass;
use image::{RgbImage, SubImage, Rgb, GenericImage};
use renderer::{self, duration_us, Renderer};

// Block types
const AIR: Block = Block::air();
const STONE: Block = Block::from_anvil_id(1 * 16);
const GRASS: Block = Block::from_anvil_id(2 * 16);
const DIRT: Block = Block::from_anvil_id(3 * 16);
const BEDROCK: Block = Block::from_anvil_id(7 * 16);
const OCEAN: Block = Block::from_anvil_id(9 * 16);
const SAND: Block = Block::from_anvil_id(12 * 16);
const GRAVEL: Block = Block::from_anvil_id(13 * 16);
const ICE: Block = Block::from_anvil_id(79 * 16);

type OverworldPasses = (ClimateSource, ShapePass, PaintPass);

#[derive(Default)]
pub struct TimeMetrics {
	pub total: u64,
	pub climates: u64,
	pub shape: u64,
	pub paint: u64,
	pub ocean: u64
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
		write!(f, "{:8.3}ms, {:5.3}ms/column | {:5.2}% / {:5.2}% / {:5.2}% / {:5.2}%",
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
	pub thread_count: u32
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
		write!(f, "{:5.2}% / {:5.2}% / {:5.2}% / {:5.2}%",
			   self.climates_percentage() / (self.thread_count as f64),
			   self.shape_percentage() / (self.thread_count as f64),
			   self.paint_percentage() / (self.thread_count as f64),
			   self.ocean_percentage() / (self.thread_count as f64)
		)
	}
}

pub fn create_renderer(seed: u64) -> FullRenderer {
	let settings = Settings::default();

	let mut biomes_config = BiomesConfig { decorator_sets: HashMap::new(), biomes: HashMap::new(), default: "plains".to_string(), grid: vec![RectConfig { temperature: (0.0, 0.1), rainfall: (0.0, 1.0), biome: "tundra".to_string() }, RectConfig { temperature: (0.1, 0.5), rainfall: (0.0, 0.2), biome: "tundra".to_string() }, RectConfig { temperature: (0.1, 0.5), rainfall: (0.2, 0.5), biome: "taiga".to_string() }, RectConfig { temperature: (0.1, 0.7), rainfall: (0.5, 1.0), biome: "swampland".to_string() }, RectConfig { temperature: (0.5, 0.95), rainfall: (0.0, 0.2), biome: "savanna".to_string() }, RectConfig { temperature: (0.5, 0.97), rainfall: (0.2, 0.35), biome: "shrubland".to_string() }, RectConfig { temperature: (0.5, 0.97), rainfall: (0.35, 0.5), biome: "forest".to_string() }, RectConfig { temperature: (0.7, 0.97), rainfall: (0.5, 1.0), biome: "forest".to_string() }, RectConfig { temperature: (0.95, 1.0), rainfall: (0.0, 0.2), biome: "desert".to_string() }, RectConfig { temperature: (0.97, 1.0), rainfall: (0.2, 0.45), biome: "plains".to_string() }, RectConfig { temperature: (0.97, 1.0), rainfall: (0.45, 0.9), biome: "seasonal_forest".to_string() }, RectConfig { temperature: (0.97, 1.0), rainfall: (0.9, 1.0), biome: "rainforest".to_string() }] };
	biomes_config.biomes.insert("seasonal_forest".to_string(), BiomeConfig { debug_name: "Seasonal Forest".to_string(), surface: SurfaceConfig { top: "2:0".to_string(), fill: "3:0".to_string(), chain: vec![] }, decorators: vec![] });
	biomes_config.biomes.insert("swampland".to_string(), BiomeConfig { debug_name: "Swampland".to_string(), surface: SurfaceConfig { top: "2:0".to_string(), fill: "3:0".to_string(), chain: vec![] }, decorators: vec![] });
	biomes_config.biomes.insert("rainforest".to_string(), BiomeConfig { debug_name: "Rainforest".to_string(), surface: SurfaceConfig { top: "2:0".to_string(), fill: "3:0".to_string(), chain: vec![] }, decorators: vec![] });
	biomes_config.biomes.insert("desert".to_string(), BiomeConfig { debug_name: "Desert".to_string(), surface: SurfaceConfig { top: "12:0".to_string(), fill: "12:0".to_string(), chain: vec![FollowupConfig { block: "24:0".to_string(), max_depth: 3 }] }, decorators: vec![] });
	biomes_config.biomes.insert("savanna".to_string(), BiomeConfig { debug_name: "Savanna".to_string(), surface: SurfaceConfig { top: "2:0".to_string(), fill: "3:0".to_string(), chain: vec![] }, decorators: vec![] });
	biomes_config.biomes.insert("plains".to_string(), BiomeConfig { debug_name: "Plains".to_string(), surface: SurfaceConfig { top: "2:0".to_string(), fill: "3:0".to_string(), chain: vec![] }, decorators: vec![] });
	biomes_config.biomes.insert("tundra".to_string(), BiomeConfig { debug_name: "Tundra".to_string(), surface: SurfaceConfig { top: "2:0".to_string(), fill: "3:0".to_string(), chain: vec![] }, decorators: vec![] });
	biomes_config.biomes.insert("shrubland".to_string(), BiomeConfig { debug_name: "Shrubland".to_string(), surface: SurfaceConfig { top: "2:0".to_string(), fill: "3:0".to_string(), chain: vec![] }, decorators: vec![] });
	biomes_config.biomes.insert("taiga".to_string(), BiomeConfig { debug_name: "Taiga".to_string(), surface: SurfaceConfig { top: "2:0".to_string(), fill: "3:0".to_string(), chain: vec![] }, decorators: vec![] });
	biomes_config.biomes.insert("forest".to_string(), BiomeConfig { debug_name: "Forest".to_string(), surface: SurfaceConfig { top: "2:0".to_string(), fill: "3:0".to_string(), chain: vec![] }, decorators: vec![] });
	biomes_config.biomes.insert("ice_desert".to_string(), BiomeConfig { debug_name: "Ice Desert".to_string(), surface: SurfaceConfig { top: "12:0".to_string(), fill: "12:0".to_string(), chain: vec![FollowupConfig { block: "24:0".to_string(), max_depth: 3 }] }, decorators: vec![] });

	//println!("{:?}", biomes_config);

	let grid = biomes_config.to_grid().unwrap();

	let ocean = OceanPass {
		blocks: OceanBlocks {
			ocean: Block::from_anvil_id(9 * 16),
			air: Block::air(),
			ice: Block::from_anvil_id(79 * 16)
		},
		sea_top: (settings.sea_coord + 1) as usize
	};

	let passes = overworld_173::passes(seed, settings, Lookup::generate(&grid));

	FullRenderer {
		passes,
		ocean
	}
}

pub struct FullRenderer {
	passes: OverworldPasses,
	ocean: OceanPass
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
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air()),
				ChunkIndexed::<Block>::new(4, Block::air())
			];

			let mut column: ColumnMut<Block> = ColumnMut::from_array(&mut column_chunks);

			let pass_start = ::std::time::Instant::now();
			let climates = climates.chunk((
				(column_position.x() * 16) as f64,
				(column_position.z() * 16) as f64
			));
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
				16
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
	fn render_column(&self, column: &ColumnMut<Block>, mut target: SubImage<&mut RgbImage>, climates: &Layer<Climate>) {
		for layer_position in LayerPosition::enumerate() {
			let mut height = 0;
			let mut ocean_height = 0;
			let mut ice = false;

			for cy in (0..128).rev() {
				let mut column_position = ColumnPosition::from_layer(cy, layer_position);
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
				AIR => Rgb { data: [255, 255, 255] },
				STONE => Rgb { data: [127, 127, 127] },
				GRASS => colorize_grass(climate),
				DIRT => Rgb { data: [255, 196, 127] },
				BEDROCK => Rgb { data: [0, 0, 0] },
				SAND => Rgb { data: [255, 240, 127] },
				GRAVEL => Rgb { data: [196, 196, 196] },
				_ => {
					println!("warning: unknown block: {:?}", top);
					no_shade = true;

					Rgb { data: [255, 0, 255] }
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
						data: [
							(color.data[0] as f64 * (1.0 - shade) * 0.5) as u8,
							(color.data[1] as f64 * (1.0 - shade) * 0.5) as u8,
							math::lerp(color.data[2] as f64, 255.0, shade) as u8
						]
					}
				} else {
					Rgb {
						data: [
							math::lerp(color.data[1] as f64 * 0.5 + 63.0, 63.0, shade) as u8,
							math::lerp(color.data[1] as f64 * 0.5 + 63.0, 63.0, shade) as u8,
							math::lerp(color.data[2] as f64, 255.0, shade) as u8
						]
					}
				}
			} else {
				let shade = math::clamp(((height as f64) / 127.0) * 0.6 + 0.4, 0.0, 1.0) ;

				let (color, shade) = if climate.freezing() {
					(Rgb { data: [255, 255, 255] }, 1.0 - (1.0 - shade).powi(2))
				} else {
					(color, shade)
				};

				Rgb {
					data: [
						(color.data[0] as f64 * shade) as u8,
						(color.data[1] as f64 * shade) as u8,
						(color.data[2] as f64 * shade) as u8
					]
				}
			};

			target.put_pixel(
				layer_position.x() as u32,
				layer_position.z() as u32,
				shaded_color
			);
		}
	}
}