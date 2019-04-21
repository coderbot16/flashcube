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

use image::{Rgb, RgbImage, SubImage, GenericImage};
use std::fs;

use i73_biome::climate::Climate;
use i73_noise::sample::Sample;
use i73_terrain::overworld::ocean::{OceanPass, OceanBlocks};
use i73_terrain::overworld_173;
use i73_biome::Lookup;
use i73_base::{Block, Pass, Layer, math};
use vocs::indexed::ChunkIndexed;
use vocs::view::ColumnMut;
use vocs::position::{GlobalColumnPosition, ColumnPosition, LayerPosition};
use std::collections::HashMap;
use i73_terrain::overworld_173::Settings;
use frontend::config::biomes::{BiomesConfig, RectConfig, BiomeConfig, SurfaceConfig, FollowupConfig};
use i73_image::colorizer::colorize_grass;

fn main() {
	generate_full_image("world", (64, 64), (784400, 0));
}

// Block types
const AIR: Block = Block::air();
const STONE: Block = Block::from_anvil_id(1 * 16);
const GRASS: Block = Block::from_anvil_id(2 * 16);
const DIRT: Block = Block::from_anvil_id(3 * 16);
const BEDROCK: Block = Block::from_anvil_id(7 * 16);
const OCEAN: Block = Block::from_anvil_id(9 * 16);
const SAND: Block = Block::from_anvil_id(12 * 16);
const GRAVEL: Block = Block::from_anvil_id(13 * 16);

fn generate_full_image(name: &str, size: (u32, u32), offset: (u32, u32)) {
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

	println!("{:?}", biomes_config);

	let grid = biomes_config.to_grid().unwrap();

	let ocean = OceanPass {
		blocks: OceanBlocks {
			ocean: Block::from_anvil_id(9 * 16),
			air: Block::air(),
			ice: Block::from_anvil_id(79 * 16)
		},
		sea_top: (settings.sea_coord + 1) as usize
	};

	let (climates, shape, paint) = overworld_173::passes(8399452073110208023, settings, Lookup::generate(&grid));

	println!("Generating region (0, 0)");
	let gen_start = ::std::time::Instant::now();
	let mut map = RgbImage::new(size.0 * 16, size.1 * 16);

	for x in 0..size.0 {
		print!("{:.2}% ", ((x as f64) / (size.0 as f64)) * 100.0);

		{
			let end = ::std::time::Instant::now();
			let time = end.duration_since(gen_start);

			let secs = time.as_secs();
			let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

			println!("[{}us elapsed ({}us per column)]", us, if x > 0 { us / ((x as u64) * (size.1 as u64)) } else { 0 });
		}

		for z in 0..size.1 {
			let column_position = GlobalColumnPosition::new((x + offset.0) as i32, (z + offset.1) as i32);

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

			let climates = climates.chunk((
				((x + offset.0) * 16) as f64,
				((z + offset.1) * 16) as f64
			));

			shape.apply(&mut column, &climates, column_position);
			paint.apply(&mut column, &climates, column_position);
			ocean.apply(&mut column, &climates, column_position);

			render_column(&column, SubImage::new(&mut map, x * 16, z * 16, 16, 16), &climates);
		}
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(gen_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Generation done in {}us ({}us per column)", us, us / ((size.0 as u64) * (size.1 as u64)));
	}

	println!("Saving image...");
	fs::create_dir_all("out/image/").unwrap();

	map.save(format!("out/image/{}.png", name)).unwrap();
}

fn render_column(column: &ColumnMut<Block>, mut target: SubImage<&mut RgbImage>, climates: &Layer<Climate>) {
	for layer_position in LayerPosition::enumerate() {
		let mut height = 0;
		let mut ocean_height = 0;

		for cy in (0..128).rev() {
			let mut column_position = ColumnPosition::from_layer(cy, layer_position);
			let block = *column.get(column_position);

			let ocean = block == OCEAN;
			let solid = block != AIR;

			if ocean_height == 0 && ocean {
				ocean_height = cy;
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

			if !climate.freezing() {
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