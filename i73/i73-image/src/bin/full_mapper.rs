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
use i73_image::colorizer::colorize_grass;
use i73_terrain::overworld::shape::ShapePass;
use i73_terrain::overworld::paint::PaintPass;
use std::thread;
use std::cmp;
use std::sync::mpsc;

fn main() {
	// Farlands
	// generate_full_image("world", (4, 4), (784400, 0));
	generate_full_image("world", (32, 32), (0, 0));
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
const ICE: Block = Block::from_anvil_id(79 * 16);

type OverworldPasses = (ClimateSource, ShapePass, PaintPass);

fn generate_full_image(name: &str, sector_size: (u32, u32), offset: (u32, u32)) {
	println!("Generating world map...");
	let gen_start = ::std::time::Instant::now();
	let mut map = RgbImage::new(sector_size.0 * 256, sector_size.1 * 256);

	let mut sectors = sector_size.0 * sector_size.1;
	let thread_count = cmp::min(4, sectors);
	let per_sector = sectors / thread_count;
	let mut threads = Vec::with_capacity(thread_count as usize);

	let (sender, receiver) = mpsc::channel();

	for _ in 0..thread_count {
		let base = sector_size.0 * sector_size.1 - sectors;
		let allotment = cmp::min(per_sector, sectors);
		sectors -= allotment;
		let sender = sender.clone();

		let handle = thread::spawn(move || {
			let (passes, ocean) = create_generator(8399452073110208023);

			for index in 0..allotment {
				let index = index + base;
				let (x, z) = (index % sector_size.0, index / sector_size.0);

				let position = GlobalSectorPosition::new(
					x as i32 + offset.0 as i32,
					z as i32 + offset.1 as i32
				);

				let sector = process_sector(position, &passes, &ocean);

				sender.send((x, z, sector)).unwrap();
			}
		});

		threads.push(handle);
	}

	let mut recieved = 0;
	while let Ok((x, z, sector)) = receiver.recv() {
		println!("Got chunk! {}, {}", x, z);
		for iz in 0..256 {
			for ix in 0..256 {
				map.put_pixel(
					x * 256 + ix,
					z * 256 + iz,
					*sector.get_pixel(ix, iz)
				);
			}
		}

		recieved += 1;

		if recieved >= sector_size.0 * sector_size.1 {
			break;
		}
	}

	for thread in threads {
		thread.join().unwrap();
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(gen_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Generation done in {}us ({}us per sector)", us, us / ((sector_size.0 as u64) * (sector_size.1 as u64)));
	}

	println!("Saving image...");
	fs::create_dir_all("out/image/").unwrap();

	map.save(format!("out/image/{}.png", name)).unwrap();
}

fn create_generator(seed: u64) -> (OverworldPasses, OceanPass) {
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

	let passes = overworld_173::passes(seed, settings, Lookup::generate(&grid));

	(passes, ocean)
}

fn process_sector(sector_position: GlobalSectorPosition, passes: &OverworldPasses, ocean: &OceanPass) -> RgbImage {
	let (ref climates, ref shape, ref paint) = &passes;

	println!("Generating sector ({}, {})", sector_position.x(), sector_position.z());
	let gen_start = ::std::time::Instant::now();
	let mut map = RgbImage::new(256, 256);

	for layer_position in LayerPosition::enumerate() {
		/*print!("{:.2}% ", ((x as f64) / (size.0 as f64)) * 100.0);

		{
			let end = ::std::time::Instant::now();
			let time = end.duration_since(gen_start);

			let secs = time.as_secs();
			let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

			println!("[{}us elapsed ({}us per column)]", us, if x > 0 { us / ((x as u64) * (size.1 as u64)) } else { 0 });
		}*/

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

		let climates = climates.chunk((
			(column_position.x() * 16) as f64,
			(column_position.z() * 16) as f64
		));

		//metrics("initial", &column, x, z);
		shape.apply(&mut column, &climates, column_position);
		//metrics("shape", &column, x, z);
		paint.apply(&mut column, &climates, column_position);
		//metrics("paint", &column, x, z);
		ocean.apply(&mut column, &climates, column_position);
		//metrics("ocean", &column, x, z);

		let target = SubImage::new(
			&mut map,
			layer_position.x() as u32 * 16,
			layer_position.z() as u32 * 16,
			16,
			16
		);

		render_column(&column, target, &climates);
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(gen_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Generation done for sector ({}, {}) in {}us ({}us per column)", sector_position.x(), sector_position.z(), us, us / 256);
	}

	map
}

/*fn metrics(stage: &'static str, column: &ColumnMut<Block>, x: u32, z: u32) {
	if x==0 && z==0 {
		println!("Chunk palette metrics @ {}:", stage);
		for index in 0..16 {
			let bits = column.0[index].bits();
			print!("[{}]: {} bits; Palette: ", index, bits);

			for entry in column.0[index].freeze().1.iter() {
				print!("{:?} ", entry);
			}

			println!();
		}
	}
}*/

fn render_column(column: &ColumnMut<Block>, mut target: SubImage<&mut RgbImage>, climates: &Layer<Climate>) {
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