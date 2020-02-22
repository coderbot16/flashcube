extern crate clap;
extern crate i73_biome;
extern crate i73_image;
extern crate image;
extern crate num_cpus;

use clap::{App, Arg};
use std::cmp;
use std::str::FromStr;

use i73_image::colorizer::{colorize_grass, DESERT};
use i73_image::renderer::climate::{ClimateRenderer, Mapper};
use i73_image::renderer::full::create_renderer;
use i73_image::stitcher;

use i73_biome::{Grid, Lookup};

use i73_biome::climate::Climate;
use image::Rgb;

#[derive(Default)]
struct MapperOptions {
	quiet: bool,
	seed: u64,
	width: u32,
	height: u32,
	threads: u32,
	world: Option<String>,
	biome: Option<String>,
	grass: Option<String>,
}

fn parse_seed(seed: &str) -> u64 {
	let seed = if seed.starts_with('-') {
		i64::from_str(seed).map(|seed| seed as u64)
	} else {
		u64::from_str(seed)
	};

	match seed {
		Ok(number) => number,
		Err(_) => unimplemented!("cannot parse string seeds yet"),
	}
}

fn validate_number(number: String) -> Result<(), String> {
	match number.parse::<u32>() {
		Ok(x) => {
			if x == 0 {
				Err("zero values are not a valid argument".to_owned())
			} else {
				Ok(())
			}
		}
		Err(parse) => Err(parse.to_string()),
	}
}

fn main() {
	let matches = App::new("i73 Mapping Tool")
		.version("0.1.0")
		.author("coderbot16 <coderbot16@gmail.com>")
		.about("Multithreaded mapper utilizing the vocs and i73 libraries to generate huge maps at ridiculous speeds")
		.arg(Arg::with_name("seed")
			.short("s")
			.long("seed")
			.value_name("SEED")
			.help("Configures the generator's random seed")
			.takes_value(true)
			.required(true)
		)
		.arg(Arg::with_name("width")
			.short("w")
			.long("width")
			.value_name("SECTORS")
			.help("Sets the image width in sectors (256 blocks per sector)")
			.default_value("4")
			.validator(validate_number)
		)
		.arg(Arg::with_name("height")
			.short("h")
			.long("height")
			.value_name("SECTORS")
			.help("Sets the image height in sectors (256 blocks per sector)")
			.default_value("4")
			.validator(validate_number)
		)
		.arg(Arg::with_name("threads")
			.short("j")
			.long("threads")
			.value_name("COUNT")
			.long_help("Configures the number of threads to use \n\
			               Default: CPU count or number of sectors to render, whichever is smaller")
			.takes_value(true)
			.validator(validate_number)
		)
		.arg(Arg::with_name("quiet")
			.short("q")
			.long("quiet")
			.help("Reduces the console spam from generation progress indicators")
		)
		.arg(Arg::with_name("world")
			.short("f")
			.long("world")
			.value_name("OUTPUT")
			.help("Generates a relatively complete world map without decorators, this is the slowest and most detailed mapper")
			.takes_value(true)
		)
		.arg(Arg::with_name("biome")
			.short("b")
			.long("biome")
			.value_name("OUTPUT")
			.help("Generates a map of the biomes in the world using a custom palette determining biome color")
			.takes_value(true)
		)
		.arg(Arg::with_name("grass")
			.short("g")
			.long("grass")
			.value_name("OUTPUT")
			.help("Generates a map of the grass colors in the world")
			.takes_value(true)
		)
		.get_matches();

	let mut options = MapperOptions::default();

	options.quiet = matches.is_present("quiet");
	options.seed = parse_seed(matches.value_of("seed").unwrap());
	options.width =
		matches.value_of("width").map(|value| u32::from_str(value).unwrap()).unwrap_or(4);
	options.height =
		matches.value_of("height").map(|value| u32::from_str(value).unwrap()).unwrap_or(4);

	let default_threads = cmp::min(options.width * options.height, num_cpus::get() as u32);
	options.threads = matches
		.value_of("threads")
		.map(|value| u32::from_str(value).unwrap())
		.unwrap_or(default_threads);

	options.world = matches.value_of("world").map(str::to_owned);
	options.biome = matches.value_of("biome").map(str::to_owned);
	options.grass = matches.value_of("grass").map(str::to_owned);

	execute(options);
}

fn execute(options: MapperOptions) {
	println!(
		"[=======] Configured image size: {} sectors x {} sectors ({} pixels x {} pixels)",
		options.width,
		options.height,
		options.width * 256,
		options.height * 256
	);

	println!(
		"[=======] Rendering images using {} thread(s) with a seed of {}",
		options.threads, options.seed
	);

	let mut any_mappers = false;

	let MapperOptions { quiet, width, height, threads, seed, world, biome, grass } = options;

	if let Some(world) = world {
		any_mappers = true;

		stitcher::generate_stitched_image(
			move || create_renderer(seed),
			world,
			(width, height),
			(0, 0),
			threads,
			quiet,
		);
	}

	if let Some(biome) = biome {
		any_mappers = true;

		execute_biome(seed, biome, (width, height), (0, 0), threads, quiet);
	}

	if let Some(grass) = grass {
		any_mappers = true;

		stitcher::generate_stitched_image(
			move || ClimateRenderer::new(seed, colorize_grass),
			grass,
			(width, height),
			(0, 0),
			threads,
			quiet,
		);
	}

	if !any_mappers {
		println!("error: no mappers specified");
		println!("help: specity a mapper with --world, --grass, or --biome");
	}
}

fn execute_biome(
	seed: u64, name: String, sector_size: (u32, u32), offset: (u32, u32), thread_count: u32,
	quiet: bool,
) {
	#[derive(Copy, Clone)]
	enum Biome {
		Tundra,
		Taiga,
		Swampland,
		Savanna,
		Shrubland,
		Forest,
		Desert,
		Plains,
		SeasonalForest,
		Rainforest,
	}

	impl Biome {
		fn color(self) -> Rgb<u8> {
			match self {
				Biome::Tundra => Rgb { data: [245, 255, 255] },
				Biome::Taiga => Rgb { data: [175, 255, 255] },
				Biome::Swampland => Rgb { data: [40, 70, 40] },
				Biome::Savanna => DESERT,
				Biome::Shrubland => Rgb { data: [150, 185, 50] },
				Biome::Forest => Rgb { data: [70, 185, 50] },
				Biome::Desert => Rgb { data: [255, 240, 127] },
				Biome::Plains => Rgb { data: [150, 220, 90] },
				Biome::SeasonalForest => Rgb { data: [70, 220, 50] },
				Biome::Rainforest => Rgb { data: [70, 255, 50] },
			}
		}
	}

	let mut grid = Grid::new(Biome::Plains);
	grid.add((0.0, 0.1), (0.0, 1.0), Biome::Tundra);
	grid.add((0.1, 0.5), (0.0, 0.2), Biome::Tundra);
	grid.add((0.1, 0.5), (0.2, 0.5), Biome::Taiga);
	grid.add((0.1, 0.7), (0.5, 1.0), Biome::Swampland);
	grid.add((0.5, 0.95), (0.0, 0.2), Biome::Savanna);
	grid.add((0.5, 0.97), (0.2, 0.35), Biome::Shrubland);
	grid.add((0.5, 0.97), (0.35, 0.5), Biome::Forest);
	grid.add((0.7, 0.97), (0.5, 1.0), Biome::Forest);
	grid.add((0.95, 1.0), (0.0, 0.2), Biome::Desert);
	grid.add((0.97, 1.0), (0.2, 0.45), Biome::Plains);
	grid.add((0.97, 1.0), (0.45, 0.9), Biome::SeasonalForest);
	grid.add((0.97, 1.0), (0.9, 1.0), Biome::Rainforest);

	struct BiomeMapper(Lookup<Biome>);

	impl Mapper for BiomeMapper {
		fn map(&self, climate: Climate) -> Rgb<u8> {
			self.0.lookup(climate).color()
		}
	}

	stitcher::generate_stitched_image(
		move || ClimateRenderer::new(seed, BiomeMapper(Lookup::generate(&grid))),
		name,
		sector_size,
		offset,
		thread_count,
		quiet,
	);
}
