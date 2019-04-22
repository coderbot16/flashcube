extern crate clap;
extern crate num_cpus;
extern crate i73_image;

use clap::{Arg, App};
use std::cmp;
use std::str::FromStr;

use i73_image::stitcher;
use i73_image::renderer::full::create_renderer;
use i73_image::renderer::climate::ClimateRenderer;
use i73_image::colorizer::colorize_grass;

#[derive(Default)]
struct MapperOptions {
	quiet: bool,
	seed: u64,
	width: u32,
	height: u32,
	threads: u32,
	world: Option<String>,
	biome: Option<String>,
	grass: Option<String>
}

fn parse_seed(seed: &str) -> u64 {
	let seed = if seed.starts_with('-') {
		i64::from_str(seed).map(|seed| seed as u64)
	} else {
		u64::from_str(seed)
	};

	match seed {
		Ok(number) => number,
		Err(_) => {
			unimplemented!("cannot parse string seeds yet")
		}
	}
}

fn validate_number(number: String) -> Result<(), String> {
	match number.parse::<u32>() {
		Ok(x) => if x == 0 {
			Err("zero values are not a valid argument".to_owned())
		} else {
			Ok(())
		},
		Err(parse) => Err(parse.to_string())
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
			.value_name("PIXELS")
			.help("Sets the image width in sectors (256 blocks per sector)")
			.default_value("4")
			.validator(validate_number)
		)
		.arg(Arg::with_name("height")
			.short("h")
			.long("height")
			.value_name("PIXELS")
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
	options.width = matches.value_of("width").map(|value| u32::from_str(value).unwrap()).unwrap_or(4);
	options.height = matches.value_of("height").map(|value| u32::from_str(value).unwrap()).unwrap_or(4);

	let default_threads = cmp::min(options.width * options.height, num_cpus::get() as u32);
	options.threads = matches.value_of("threads").map(|value| u32::from_str(value).unwrap()).unwrap_or(default_threads);

	options.world = matches.value_of("world").map(str::to_owned);
	options.biome = matches.value_of("biome").map(str::to_owned);
	options.grass = matches.value_of("grass").map(str::to_owned);

	execute(options);
}

fn execute(options: MapperOptions) {
	println!("[=======] Configured image size: {} sectors x {} sectors ({} pixels x {} pixels)",
		options.width,
		options.height,
		options.width * 256,
		options.height * 256
	);

	println!("[=======] Rendering images using {} thread(s) with a seed of {}",
		options.threads,
		options.seed
	);

	let mut any_mappers = false;

	let MapperOptions {
		quiet,
		width,
		height,
		threads,
		seed,
		world,
		biome,
		grass
	} = options;

	if let Some(world) = world {
		any_mappers = true;

		stitcher::generate_stitched_image(move || { create_renderer(seed) }, world, (width, height), (0, 0), threads, quiet);
	}

	if let Some(biome) = biome {
		any_mappers = true;

		println!("error: the biome mapper is not currently implemented, map {} not rendered", biome);
	}

	if let Some(grass) = grass {
		any_mappers = true;

		stitcher::generate_stitched_image(move || { ClimateRenderer::new(seed, colorize_grass) }, grass, (width, height), (0, 0), threads, quiet);
	}

	if !any_mappers {
		println!("error: no mappers specified");
		println!("help: specity a mapper with --world, --grass, or --biome");
	}
}