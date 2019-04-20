extern crate image;
extern crate i73_base;
extern crate i73_image;
extern crate i73_biome;
extern crate i73_noise;
extern crate i73_shape;
extern crate i73_terrain;
extern crate frontend;
extern crate cgmath;

extern crate java_rand;
extern crate vocs;

use image::{Rgb, RgbImage};
use std::fs;

//use i73_image::colorizer::colorize_grass;
use i73_biome::climate::{ClimateSource, ClimateSettings};
use i73_noise::sample::Sample;
use i73_shape::height::{HeightSource, HeightSettings, HeightSettings81};
use i73_noise::octaves::PerlinOctaves;
use cgmath::{Point2, Vector3};
use i73_shape::volume::TriNoiseSettings;
use i73_terrain::overworld::ocean::{OceanPass, OceanBlocks};
use i73_terrain::overworld_173;
use i73_biome::Lookup;
use i73_base::{Block, Pass, math};
use i73_base::matcher::BlockMatcher;
use vocs::indexed::ChunkIndexed;
use vocs::view::ColumnMut;
use vocs::position::{GlobalColumnPosition, ColumnPosition};
use std::collections::HashMap;
use i73_terrain::overworld_173::Settings;
use frontend::config::biomes::{BiomesConfig, RectConfig, BiomeConfig, SurfaceConfig, FollowupConfig};
use i73_image::colorizer::colorize_grass;

fn main() {
	// Initialization
	let seed = 8399452073110208023;

	let mut rng = java_rand::Random::new(seed);

	let _tri = i73_shape::volume::TriNoiseSource::new(&mut rng, &TriNoiseSettings::default());

	let _sand      = PerlinOctaves::new(&mut rng.clone(), 4, Vector3::new(1.0 / 32.0, 1.0 / 32.0,        1.0)); // Vertical,   Z =   0.0
	let _gravel    = PerlinOctaves::new(&mut rng,                        4, Vector3::new(1.0 / 32.0,        1.0, 1.0 / 32.0)); // Horizontal
	let _thickness = PerlinOctaves::new(&mut rng,                        4, Vector3::new(1.0 / 16.0, 1.0 / 16.0, 1.0 / 16.0)); // Vertical,   Z =   0.0

	let height_source  = HeightSource::new(&mut rng, &HeightSettings::default());
	let climates = ClimateSource::new(seed, ClimateSettings::default());

	// Image generation

	generate_full_image("world", (256, 256));
}

fn generate_full_image(name: &str, size: (u32, u32)) {
	let mut settings = Settings::default();

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

	println!("{:?}", biomes_config);

	let grid = biomes_config.to_grid().unwrap();

	let climates = ClimateSource::new(8399452073110208023, settings.climate);
	let ocean = OceanPass {
		climate: ClimateSource::new(8399452073110208023, settings.climate),
		blocks: OceanBlocks {
			ocean: settings.paint_blocks.ocean.clone(),
			air: settings.paint_blocks.air.clone()
		},
		sea_top: (settings.sea_coord + 1) as usize
	};

	let (shape, paint) = overworld_173::passes(8399452073110208023, settings, Lookup::generate(&grid));

	/*let caves_generator = i73_structure::caves::CavesGenerator {
		carve: Block::air(),
		lower: Block::from_anvil_id(10*16),
		surface_block: Block::from_anvil_id(2*16),
		ocean: BlockMatcher::include([Block::from_anvil_id(8*16), Block::from_anvil_id(9*16)].iter()),
		carvable: BlockMatcher::include([Block::from_anvil_id(1*16), Block::from_anvil_id(2*16), Block::from_anvil_id(3*16)].iter()),
		surface_top: BlockMatcher::is(Block::from_anvil_id(2*16)),
		surface_fill: BlockMatcher::is(Block::from_anvil_id(3*16)),
		blob_size_multiplier: 1.0,
		vertical_multiplier: 1.0,
		lower_surface: 10
	};
	let caves = i73_structure::StructureGenerateNearby::new(8399452073110208023, 8, caves_generator);*/

	println!("Generating region (0, 0)");
	let gen_start = ::std::time::Instant::now();
	let mut map = RgbImage::new(size.0 * 16, size.1 * 16);

	for x in 0..size.0 {
		println!("{:.2}%", ((x as f64) / (size.0 as f64)) * 100.0);

		for z in 0..size.1 {
			let column_position = GlobalColumnPosition::new(x as i32, z as i32);

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

			shape.apply(&mut column, column_position);
			paint.apply(&mut column, column_position);
			ocean.apply(&mut column, column_position);
			//caves.apply(&mut column, column_position);

			for cz in 0..16 {
				for cx in 0..16 {
					let mut height = 0;
					let mut ocean_height = 0;

					for cy in (0..128).rev() {
						let mut column_position = ColumnPosition::new(cx, cy, cz);
						let block = *column.get(column_position);

						let ocean = block == Block::from_anvil_id(9 * 16);
						let solid = block != Block::air();

						if ocean_height == 0 && ocean {
							ocean_height = cy;
							continue;
						}

						if solid && !ocean {
							height = cy;
							break;
						}
					}

					let shade = (height as f64) / 127.0;
					let position = ColumnPosition::new(cx, height, cz);
					let top = *column.get(position);

					// Block types
					const AIR: Block = Block::air();
					const STONE: Block = Block::from_anvil_id(1 * 16);
					const GRASS: Block = Block::from_anvil_id(2 * 16);
					const DIRT: Block = Block::from_anvil_id(3 * 16);
					const OCEAN: Block = Block::from_anvil_id(9 * 16);
					const SAND: Block = Block::from_anvil_id(12 * 16);
					const GRAVEL: Block = Block::from_anvil_id(13 * 16);

					let color = match top {
						AIR => Rgb { data: [255, 255, 255] },
						STONE => Rgb { data: [127, 127, 127] },
						GRASS => colorize_grass(climates.sample(Point2::new(
							(x * 16 + cx as u32) as f64,
							(z * 16 + cz as u32) as f64
						))),
						DIRT => Rgb { data: [255, 196, 127] },
						SAND => Rgb { data: [255, 240, 127] },
						GRAVEL => Rgb { data: [196, 196, 196] },
						_ => Rgb { data: [255, 0, 255] }
					};

					let shaded_color = if ocean_height != 0 {
						let depth = ocean_height - height;
						let shade = math::clamp(depth as f64 / 10.0, 0.0, 1.0);
						let shade = 1.0 - (1.0 - shade).powi(2);

						Rgb {
							data: [
								(color.data[0] as f64 * (1.0 - shade) * 0.5) as u8,
								(color.data[1] as f64 * (1.0 - shade) * 0.5) as u8,
								math::lerp(color.data[2] as f64, 255f64, shade) as u8
							]
						}
					} else {
						Rgb {
							data: [
								(color.data[0] as f64 * shade) as u8,
								(color.data[1] as f64 * shade) as u8,
								(color.data[2] as f64 * shade) as u8
							]
						}
					};

					map.put_pixel(
						x * 16 + cx as u32,
						z * 16 + cz as u32,
						shaded_color
					);
				}
			}
		}
	}

	println!("Generation complete, saving image...");
	fs::create_dir_all("out/image/").unwrap();

	map.save(format!("out/image/{}.png", name)).unwrap();
}