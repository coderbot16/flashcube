extern crate cgmath;
extern crate deflate;
extern crate frontend as i73;
extern crate i73_base;
extern crate i73_biome;
extern crate i73_decorator;
extern crate i73_noise;
extern crate i73_shape;
extern crate i73_structure;
extern crate i73_terrain;
extern crate java_rand;
extern crate nbt_turbo;
extern crate rs25;
extern crate vocs;

use std::cmp::min;
use std::fs::File;
use std::path::PathBuf;

use i73::config::biomes::{BiomeConfig, BiomesConfig, FollowupConfig, RectConfig, SurfaceConfig};
use i73::config::settings::customized::{
	BiomeSettings, Decorators, Ocean, Parts, Structures, VeinSettings, VeinSettingsCentered,
};
use i73_base::matcher::BlockMatcher;
use i73_base::{Block, Layer, Pass};
use i73_biome::Lookup;
use i73_terrain::overworld::ocean::{OceanBlocks, OceanPass};
use i73_terrain::overworld_173::{self, Settings};

use cgmath::Vector3;

use vocs::indexed::ChunkIndexed;
use vocs::position::{
	ChunkPosition, GlobalChunkPosition, GlobalColumnPosition, LayerPosition, QuadPosition,
};
use vocs::view::ColumnMut;
use vocs::world::world::World;

use deflate::Compression;
use i73_decorator::tree::{LargeTreeDecorator, NormalTreeDecorator};
use i73_decorator::Decorator;
use i73_noise::sample::Sample;
use i73_shape::height::HeightSettings81;
use i73_shape::volume::TriNoiseSettings;
use std::collections::HashMap;
use vocs::nibbles::ChunkNibbles;
use vocs::world::shared::{NoPack, SharedWorld};
use vocs::position::{dir, Offset};

use i73::lighting;

fn main() {
	let main_start = ::std::time::Instant::now();

	let profile_name = match ::std::env::args().skip(1).next() {
		Some(name) => name,
		None => {
			println!("Usage: i73 <profile>");
			return;
		}
	};

	let mut profile = PathBuf::new();
	profile.push("profiles");
	profile.push(&profile_name);

	println!("Using profile {}: {}", profile_name, profile.to_string_lossy());

	// TODO: Better JSON parser; uncommenting this adds 9 seconds to the compile time
	/*let customized = serde_json::from_reader::<File, Customized>(File::open(profile.join("customized.json")).unwrap()).unwrap();
	let parts = Parts::from(customized);*/

	let parts = Parts {
		tri: TriNoiseSettings {
			main_out_scale: 20.0,
			upper_out_scale: 512.0,
			lower_out_scale: 512.0,
			lower_scale: Vector3 { x: 684.412, y: 684.412, z: 684.412 },
			upper_scale: Vector3 { x: 684.412, y: 684.412, z: 684.412 },
			main_scale: Vector3 { x: 8.55515, y: 4.277575, z: 8.55515 },
			y_size: 17,
		},
		height_stretch: 12.0,
		height: HeightSettings81 {
			coord_scale: Vector3 { x: 200.0, y: 0.0, z: 200.0 },
			out_scale: 8000.0,
			base: 8.5,
		},
		biome: BiomeSettings {
			depth_weight: 1.0,
			depth_offset: 0.0,
			scale_weight: 1.0,
			scale_offset: 0.0,
			fixed: -1,
			biome_size: 4,
			river_size: 4,
		},
		ocean: Ocean { top: 64, lava: false },
		structures: Structures {
			caves: true,
			strongholds: true,
			villages: true,
			mineshafts: true,
			temples: true,
			ravines: true,
		},
		decorators: Decorators {
			dungeon_chance: Some(8),
			water_lake_chance: Some(4),
			lava_lake_chance: Some(80),
			dirt: VeinSettings { size: 33, count: 10, min_y: 0, max_y: 256 },
			gravel: VeinSettings { size: 33, count: 8, min_y: 0, max_y: 256 },
			granite: VeinSettings { size: 33, count: 10, min_y: 0, max_y: 80 },
			diorite: VeinSettings { size: 33, count: 10, min_y: 0, max_y: 80 },
			andesite: VeinSettings { size: 33, count: 10, min_y: 0, max_y: 80 },
			coal: VeinSettings { size: 17, count: 20, min_y: 0, max_y: 128 },
			iron: VeinSettings { size: 9, count: 20, min_y: 0, max_y: 64 },
			redstone: VeinSettings { size: 8, count: 8, min_y: 0, max_y: 16 },
			diamond: VeinSettings { size: 8, count: 1, min_y: 0, max_y: 16 },
			lapis: VeinSettingsCentered { size: 7, count: 1, center_y: 16, spread: 16 },
		},
	};

	println!("  Tri Noise Settings: {:?}", parts.tri);
	println!("  Height Stretch: {:?}", parts.height_stretch);
	println!("  Height Settings: {:?}", parts.height);
	println!("  Biome Settings: {:?}", parts.biome);
	println!("  Structures: {:?}", parts.structures);
	println!("  Decorators: {:?}", parts.decorators);

	let mut settings = Settings::default();

	settings.tri = parts.tri;
	settings.height = parts.height.into();
	settings.field.height_stretch = parts.height_stretch;

	// TODO: Biome Settings
	println!();
	let sea_block = Block::from_anvil_id(if parts.ocean.top > 0 {
		settings.sea_coord = min(parts.ocean.top - 1, 255) as u8;

		if parts.ocean.lava {
			11 * 16
		} else {
			9 * 16
		}
	} else {
		0 * 16
	});

	// TODO: Structures and Decorators

	// TODO: Better JSON parser; uncommenting this adds 18 seconds to the compile time
	// let biomes_config = serde_json::from_reader::<File, BiomesConfig>(File::open(profile.join("biomes.json")).unwrap()).unwrap();
	let mut biomes_config = BiomesConfig {
		decorator_sets: HashMap::new(),
		biomes: HashMap::new(),
		default: "plains".to_string(),
		grid: vec![
			RectConfig {
				temperature: (0.0, 0.1),
				rainfall: (0.0, 1.0),
				biome: "tundra".to_string(),
			},
			RectConfig {
				temperature: (0.1, 0.5),
				rainfall: (0.0, 0.2),
				biome: "tundra".to_string(),
			},
			RectConfig {
				temperature: (0.1, 0.5),
				rainfall: (0.2, 0.5),
				biome: "taiga".to_string(),
			},
			RectConfig {
				temperature: (0.1, 0.7),
				rainfall: (0.5, 1.0),
				biome: "swampland".to_string(),
			},
			RectConfig {
				temperature: (0.5, 0.95),
				rainfall: (0.0, 0.2),
				biome: "savanna".to_string(),
			},
			RectConfig {
				temperature: (0.5, 0.97),
				rainfall: (0.2, 0.35),
				biome: "shrubland".to_string(),
			},
			RectConfig {
				temperature: (0.5, 0.97),
				rainfall: (0.35, 0.5),
				biome: "forest".to_string(),
			},
			RectConfig {
				temperature: (0.7, 0.97),
				rainfall: (0.5, 1.0),
				biome: "forest".to_string(),
			},
			RectConfig {
				temperature: (0.95, 1.0),
				rainfall: (0.0, 0.2),
				biome: "desert".to_string(),
			},
			RectConfig {
				temperature: (0.97, 1.0),
				rainfall: (0.2, 0.45),
				biome: "plains".to_string(),
			},
			RectConfig {
				temperature: (0.97, 1.0),
				rainfall: (0.45, 0.9),
				biome: "seasonal_forest".to_string(),
			},
			RectConfig {
				temperature: (0.97, 1.0),
				rainfall: (0.9, 1.0),
				biome: "rainforest".to_string(),
			},
		],
	};
	biomes_config.biomes.insert(
		"seasonal_forest".to_string(),
		BiomeConfig {
			debug_name: "seasonal_forest".to_string(),
			surface: SurfaceConfig {
				top: "2:0".to_string(),
				fill: "3:0".to_string(),
				chain: vec![],
			},
			decorators: vec![],
		},
	);
	biomes_config.biomes.insert(
		"swampland".to_string(),
		BiomeConfig {
			debug_name: "swampland".to_string(),
			surface: SurfaceConfig {
				top: "2:0".to_string(),
				fill: "3:0".to_string(),
				chain: vec![],
			},
			decorators: vec![],
		},
	);
	biomes_config.biomes.insert(
		"rainforest".to_string(),
		BiomeConfig {
			debug_name: "rainforest".to_string(),
			surface: SurfaceConfig {
				top: "2:0".to_string(),
				fill: "3:0".to_string(),
				chain: vec![],
			},
			decorators: vec![],
		},
	);
	biomes_config.biomes.insert(
		"desert".to_string(),
		BiomeConfig {
			debug_name: "desert".to_string(),
			surface: SurfaceConfig {
				top: "12:0".to_string(),
				fill: "12:0".to_string(),
				chain: vec![FollowupConfig { block: "24:0".to_string(), max_depth: 3 }],
			},
			decorators: vec![],
		},
	);
	biomes_config.biomes.insert(
		"savanna".to_string(),
		BiomeConfig {
			debug_name: "savanna".to_string(),
			surface: SurfaceConfig {
				top: "2:0".to_string(),
				fill: "3:0".to_string(),
				chain: vec![],
			},
			decorators: vec![],
		},
	);
	biomes_config.biomes.insert(
		"plains".to_string(),
		BiomeConfig {
			debug_name: "plains".to_string(),
			surface: SurfaceConfig {
				top: "2:0".to_string(),
				fill: "3:0".to_string(),
				chain: vec![],
			},
			decorators: vec![],
		},
	);
	biomes_config.biomes.insert(
		"tundra".to_string(),
		BiomeConfig {
			debug_name: "tundra".to_string(),
			surface: SurfaceConfig {
				top: "2:0".to_string(),
				fill: "3:0".to_string(),
				chain: vec![],
			},
			decorators: vec![],
		},
	);
	biomes_config.biomes.insert(
		"shrubland".to_string(),
		BiomeConfig {
			debug_name: "shrubland".to_string(),
			surface: SurfaceConfig {
				top: "2:0".to_string(),
				fill: "3:0".to_string(),
				chain: vec![],
			},
			decorators: vec![],
		},
	);
	biomes_config.biomes.insert(
		"taiga".to_string(),
		BiomeConfig {
			debug_name: "taiga".to_string(),
			surface: SurfaceConfig {
				top: "2:0".to_string(),
				fill: "3:0".to_string(),
				chain: vec![],
			},
			decorators: vec![],
		},
	);
	biomes_config.biomes.insert(
		"forest".to_string(),
		BiomeConfig {
			debug_name: "forest".to_string(),
			surface: SurfaceConfig {
				top: "2:0".to_string(),
				fill: "3:0".to_string(),
				chain: vec![],
			},
			decorators: vec![],
		},
	);
	biomes_config.biomes.insert(
		"ice_desert".to_string(),
		BiomeConfig {
			debug_name: "ice_desert".to_string(),
			surface: SurfaceConfig {
				top: "12:0".to_string(),
				fill: "12:0".to_string(),
				chain: vec![FollowupConfig { block: "24:0".to_string(), max_depth: 3 }],
			},
			decorators: vec![],
		},
	);

	println!("{:?}", biomes_config);

	let grid = biomes_config.to_grid().unwrap();

	/*let mut decorator_registry: ::std::collections::HashMap<String, Box<i73::config::decorator::DecoratorFactory>> = ::std::collections::HashMap::new();
	decorator_registry.insert("vein".into(), Box::new(::i73::config::decorator::vein::VeinDecoratorFactory::default()));
	decorator_registry.insert("seaside_vein".into(), Box::new(::i73::config::decorator::vein::SeasideVeinDecoratorFactory::default()));
	decorator_registry.insert("lake".into(), Box::new(::i73::config::decorator::lake::LakeDecoratorFactory::default()));*/

	/*let gravel_config = DecoratorConfig {
		decorator: "vein".into(),
		settings: json!({
			"blocks": {
				"replace": {
					"blacklist": false,
					"blocks": [16]
				},
				"block": 208
			},
			"size": 32
		}),
		height_distribution: ::i73_base::distribution::Chance {
			base: i73_base::distribution::Baseline::Linear(i73_base::distribution::Linear {
				min: 0,
				max: 63
			}),
			ordering: i73_base::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		},
		count: ::i73_base::distribution::Chance {
			base: i73_base::distribution::Baseline::Linear(i73_base::distribution::Linear {
				min: 0,
				max: 9
			}),
			ordering: i73_base::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		}
	};*/

	let mut decorators: Vec<
		::i73_decorator::Dispatcher<
			i73_base::distribution::Chance<i73_base::distribution::Baseline>,
			i73_base::distribution::Chance<i73_base::distribution::Baseline>,
		>,
	> = Vec::new();

	/*for (name, decorator_set) in biomes_config.decorator_sets {
		println!("Configuring decorator set {}", name);

		for decorator_config in decorator_set {
			println!("Config: {:?}", decorator_config);

			decorators.push(decorator_config.into_dispatcher(&decorator_registry).unwrap());
		}
	}*/

	decorators.push(::i73_decorator::Dispatcher {
		decorator: Box::new(::i73_decorator::lake::LakeDecorator {
			blocks: ::i73_decorator::lake::LakeBlocks {
				is_liquid: BlockMatcher::include(
					[
						Block::from_anvil_id(8 * 16),
						Block::from_anvil_id(9 * 16),
						Block::from_anvil_id(10 * 16),
						Block::from_anvil_id(11 * 16),
					]
					.iter(),
				),
				is_solid: BlockMatcher::exclude(
					[
						Block::air(),
						Block::from_anvil_id(8 * 16),
						Block::from_anvil_id(9 * 16),
						Block::from_anvil_id(10 * 16),
						Block::from_anvil_id(11 * 16),
					]
					.iter(),
				), // TODO: All nonsolid blocks
				replaceable: BlockMatcher::none(), // TODO
				liquid: Block::from_anvil_id(9 * 16),
				carve: Block::air(),
				solidify: None,
			},
			settings: ::i73_decorator::lake::LakeSettings::default(),
		}),
		height_distribution: ::i73_base::distribution::Chance {
			base: i73_base::distribution::Baseline::Linear(i73_base::distribution::Linear {
				min: 0,
				max: 127,
			}),
			ordering: i73_base::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1,
		},
		rarity: ::i73_base::distribution::Chance {
			base: ::i73_base::distribution::Baseline::Constant { value: 1 },
			chance: 4,
			ordering: ::i73_base::distribution::ChanceOrdering::AlwaysGeneratePayload,
		},
	});

	/*decorators.push (::i73_decorator::Dispatcher {
		decorator: Box::new(::i73_decorator::vein::SeasideVeinDecorator {
			vein: ::i73_decorator::vein::VeinDecorator {
				blocks: ::i73_decorator::vein::VeinBlocks {
					replace: BlockMatcher::is(Block::from_anvil_id(12*16)),
					block: Block::from_anvil_id(82*16)
				},
				size: 32
			},
			ocean: BlockMatcher::include([Block::from_anvil_id(8*16), Block::from_anvil_id(9*16)].iter())
		}),
		height_distribution: ::i73_base::distribution::Chance {
			base: i73_base::distribution::Baseline::Linear(i73_base::distribution::Linear {
				min: 0,
				max: 63
			}),
			ordering: i73_base::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		},
		rarity: ::i73_base::distribution::Chance {
			base: i73_base::distribution::Baseline::Linear(i73_base::distribution::Linear {
				min: 0,
				max: 9
			}),
			ordering: i73_base::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		}
	});*/

	//decorators.push (gravel_config.into_dispatcher(&decorator_registry).unwrap());

	/*decorators.push (::i73_decorator::Dispatcher {
		decorator: Box::new(::i73_decorator::clump::Clump {
			iterations: 64,
			horizontal: 8,
			vertical: 4,
			decorator: ::i73_decorator::clump::plant::PlantDecorator {
				block: Block::from_anvil_id(31*16 + 1),
				base: BlockMatcher::include([Block::from_anvil_id(2*16), Block::from_anvil_id(3*16), Block::from_anvil_id(60*16)].into_iter()),
				replace: BlockMatcher::is(Block::air())
			}
		}),
		height_distribution: ::i73_base::distribution::Chance {
			base: i73_base::distribution::Baseline::Linear(i73_base::distribution::Linear {
				min: 0,
				max: 127
			}),
			ordering: i73_base::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		},
		rarity: ::i73_base::distribution::Chance {
			base: i73_base::distribution::Baseline::Linear(i73_base::distribution::Linear {
				min: 0,
				max: 10
			}),
			ordering: i73_base::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		}
	});*/

	/*use large_tree::{LargeTreeSettings, LargeTree};
	let settings = LargeTreeSettings::default();

	for i in 0..1 {
		let mut rng = Random::new(100 + i);
		let shape = settings.tree((0, 0, 0), &mut rng, None, 20);

		println!("{:?}", shape);

		let mut y = shape.foliage_max_y - 1;
		while y >= shape.foliage_min_y {
			let spread = shape.spread(y);

			println!("y: {}, spread: {}", y, spread);

			for _ in 0..shape.foliage_per_y {
				println!("{:?}", shape.foliage(y, spread, &mut rng));
			}

			y -= 1;
		}
	}*/

	let ocean = OceanPass {
		blocks: OceanBlocks {
			ocean: sea_block,
			air: settings.paint_blocks.air.clone(),
			ice: Block::from_anvil_id(79 * 16),
		},
		sea_top: (settings.sea_coord + 1) as usize,
	};

	let (climates, shape, paint) =
		overworld_173::passes(8399452073110208023, settings, Lookup::generate(&grid));

	let caves_generator = i73_structure::caves::CavesGenerator {
		carve: Block::air(),
		lower: Block::from_anvil_id(10 * 16),
		surface_block: Block::from_anvil_id(2 * 16),
		ocean: BlockMatcher::include(
			[Block::from_anvil_id(8 * 16), Block::from_anvil_id(9 * 16)].iter(),
		),
		carvable: BlockMatcher::include(
			[
				Block::from_anvil_id(1 * 16),
				Block::from_anvil_id(2 * 16),
				Block::from_anvil_id(3 * 16),
			]
			.iter(),
		),
		surface_top: BlockMatcher::is(Block::from_anvil_id(2 * 16)),
		surface_fill: BlockMatcher::is(Block::from_anvil_id(3 * 16)),
		spheroid_size_multiplier: 1.0,
		vertical_multiplier: 1.0,
		lower_surface: 10,
	};
	let caves =
		i73_structure::StructureGenerateNearby::new(8399452073110208023, 8, caves_generator);

	/*let shape = nether_173::passes(-160654125608861039, &nether_173::default_tri_settings(), nether_173::ShapeBlocks::default(), 31);

	let default_grid = biome::default_grid();

	let mut fake_settings = Settings::default();
	fake_settings.biome_lookup = biome::Lookup::filled(default_grid.lookup(biome::climate::Climate::new(0.5, 0.0)));
	fake_settings.sea_coord = 31;
	fake_settings.beach = None;
	fake_settings.max_bedrock_height = None;

	let (_, paint) = overworld_173::passes(-160654125608861039, fake_settings);*/

	let mut world = World::<ChunkIndexed<Block>>::new();
	let mut world_biomes = HashMap::<(i32, i32), Vec<u8>>::new();

	println!("Generating region (0, 0)");
	let gen_start = ::std::time::Instant::now();

	for x in 0..32 {
		println!("{}", x);
		for z in 0..32 {
			let column_position = GlobalColumnPosition::new(x, z);

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
				ChunkIndexed::<Block>::new(4, Block::air()),
			];

			let climate = climates
				.chunk((column_position.x() as f64 * 16.0, column_position.z() as f64 * 16.0));
			let lookup = paint.biome_lookup();
			let mut biomes = Vec::with_capacity(256);

			for position in LayerPosition::enumerate() {
				let climate = climate.get(position);
				let biome = lookup.lookup(climate);

				let id = match biome.name.as_ref() {
					"rainforest" => 21,      // jungle
					"seasonal_forest" => 23, // jungle_edge
					"forest" => 4,           // forest
					"swampland" => 3,        // mountains
					"savanna" => 35,         // savanna
					"shrubland" => 1,        // plains
					"taiga" => 30,           // cold_taiga
					"desert" => 2,           // desert
					"plains" => 1,           // plains
					"tundra" => 12,          // ice_plains
					"ice_desert" => 12,      // ice_plains
					unknown => panic!("Unknown biome {}", unknown),
				};

				biomes.push(id);
			}

			world_biomes.insert((x, z), biomes);

			{
				let mut column: ColumnMut<Block> = ColumnMut::from_array(&mut column_chunks);

				shape.apply(&mut column, &climate, column_position);
				paint.apply(&mut column, &climate, column_position);
				ocean.apply(&mut column, &climate, column_position);
				caves.apply(&mut column, &Layer::fill(()), column_position);
			}

			world.set_column(column_position, column_chunks);
		}
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(gen_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Generation done in {}us ({}us per column)", us, us / 1024);
	}

	println!("Decorating region (0, 0)");
	let dec_start = ::std::time::Instant::now();

	let mut decoration_rng = ::java_rand::Random::new(8399452073110208023);
	let coefficients =
		(((decoration_rng.next_i64() >> 1) << 1) + 1, ((decoration_rng.next_i64() >> 1) << 1) + 1);

	for x in 0..31 {
		println!("{}", x);
		for z in 0..31 {
			let x_part = (x as i64).wrapping_mul(coefficients.0) as u64;
			let z_part = (z as i64).wrapping_mul(coefficients.1) as u64;
			decoration_rng =
				::java_rand::Random::new((x_part.wrapping_add(z_part)) ^ 8399452073110208023);

			let mut quad =
				world.get_quad_mut(GlobalColumnPosition::new(x as i32, z as i32)).unwrap();

			'outer: for _ in 0..8 {
				let mut position = QuadPosition::new(
					decoration_rng.next_u32_bound(16) as u8 + 8,
					127,
					decoration_rng.next_u32_bound(16) as u8 + 8,
				);

				while quad.get(position) == &Block::air() {
					position = match position.offset(dir::Down) {
						Some(pos) => pos,
						None => break 'outer,
					};
				}

				if decoration_rng.next_bool() {
					LargeTreeDecorator::default()
						.generate(
							&mut quad,
							&mut decoration_rng,
							position.offset(dir::Up).unwrap_or(position),
						)
						.unwrap();
				} else {
					NormalTreeDecorator::default()
						.generate(
							&mut quad,
							&mut decoration_rng,
							position.offset(dir::Up).unwrap_or(position),
						)
						.unwrap();
				}
			}

			for dispatcher in &decorators {
				dispatcher.generate(&mut quad, &mut decoration_rng).unwrap();
			}
		}
	}

	/*for x in 0..31 {
		println!("{}", x);
		for z in 0..31 {
			let x_part = (x as i64).wrapping_mul(coefficients.0) as u64;
			let z_part = (z as i64).wrapping_mul(coefficients.1) as u64;
			decoration_rng = ::java_rand::Random::new((x_part.wrapping_add(z_part)) ^ 8399452073110208023);

			let mut quad = world.get_quad_mut(GlobalColumnPosition::new(x as i32, z as i32)).unwrap();
			// TODO: Biomes paint.biomes()

			for dispatcher in &decorators {
				dispatcher.generate(&mut quad, &mut decoration_rng).unwrap();
			}
		}
	}*/

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(dec_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Decoration done in {}us ({}us per column)", us, us / 1024);
	}

	// Also logs timing messages
	let (mut sky_light, mut heightmaps) = lighting::compute_skylight(&world);

	println!("Writing region (0, 0)");
	let writing_start = ::std::time::Instant::now();

	write_region(&world, &mut sky_light, &mut heightmaps, &mut world_biomes);

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(writing_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Writing done in {}us ({}us per column)", us, us / 1024);
		println!();
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(main_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("i73 done in {}us ({}us per column)", us, us / 1024);
		println!();
	}
}

fn write_classicworld(world: &World<ChunkIndexed<Block>>) {
	let mut blocks = vec![0; 512 * 128 * 512];
	for z in 0..32 {
		println!("{}", z);
		for x in 0..32 {
			let column_position = GlobalColumnPosition::new(x, z);

			for y in 0..8 {
				let chunk_position = GlobalChunkPosition::from_column(column_position, y);

				let chunk = world.get(chunk_position).unwrap();

				fn index(x: u32, y: u32, z: u32) -> u32 {
					(y * 512 + z) * 512 + x
				}

				for position in ChunkPosition::enumerate() {
					let i = index(
						position.x() as u32 + x as u32 * 16,
						position.y() as u32 + y as u32 * 16,
						position.z() as u32 + z as u32 * 16,
					);

					let mut block: u16 = (*chunk.get(position)).into();
					block /= 16;

					// Sandstone is ID 52 in ClassiCube, not 24
					if block == 24 {
						block = 52;
					}

					blocks[i as usize] = block as u8;
				}
			}
		}
	}

	use nbt_turbo::writer::CompoundWriter;
	let buffer = CompoundWriter::write("ClassicWorld", Vec::new(), |writer| {
		writer
			.u8_array("BlockArray", &blocks)
			.i8("FormatVersion", 1)
			.string("Name", "i73 Test World")
			.u8_array("UUID", &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
			.i16("X", 512)
			.i16("Y", 128)
			.i16("Z", 512)
			.i64("TimeCreated", 0)
			.i64("LastAccessed", 0)
			.i64("LastModified", 0)
			.compound("Spawn", |writer| {
				writer.i16("X", 90).i16("Y", 64).i16("Z", 90);
			});
	});

	use deflate::write::GzEncoder;
	use std::io::Write;

	let file = File::create("out/classic/i73.cw").unwrap();
	let mut gzip = GzEncoder::new(file, Compression::Fast);
	gzip.write_all(&buffer).unwrap();
	gzip.finish().unwrap();
}

fn write_region(
	world: &World<ChunkIndexed<Block>>, sky_light: &mut SharedWorld<NoPack<ChunkNibbles>>,
	heightmaps: &mut HashMap<(i32, i32), Vec<u32>>,
	world_biomes: &mut HashMap<(i32, i32), Vec<u8>>,
) {
	use rs25::level::anvil::ColumnRoot;
	use rs25::level::manager::{ChunkSnapshot, ColumnSnapshot};
	use rs25::level::region::RegionWriter;

	let file = File::create("out/region/r.0.0.mca").unwrap();
	let mut writer = RegionWriter::start(file).unwrap();

	for z in 0..32 {
		println!("{}", z);
		for x in 0..32 {
			let column_position = GlobalColumnPosition::new(x, z);

			let heightmap = heightmaps.remove(&(x, z)).unwrap();

			let mut snapshot = ColumnSnapshot {
				chunks: vec![None; 16],
				last_update: 0,
				light_populated: true,
				terrain_populated: true,
				inhabited_time: 0,
				biomes: world_biomes.remove(&(x, z)).unwrap(),
				heightmap,
				tile_ticks: vec![],
			};

			for y in 0..16 {
				let chunk_position = GlobalChunkPosition::from_column(column_position, y);

				let chunk = world.get(chunk_position).unwrap();

				/*if chunk.anvil_empty() {
					continue;
				}*/

				let sky_light = sky_light.remove(chunk_position).unwrap()/*_or_else(ChunkNibbles::default)*/;

				snapshot.chunks[y as usize] = Some(ChunkSnapshot {
					blocks: chunk.clone(),
					block_light: ChunkNibbles::default(),
					sky_light: sky_light.0,
				});
			}

			let root = ColumnRoot::from(snapshot.to_column(x as i32, z as i32).unwrap());

			writer.column(x as u8, z as u8, &root).unwrap();
		}
	}

	writer.finish().unwrap();
}
