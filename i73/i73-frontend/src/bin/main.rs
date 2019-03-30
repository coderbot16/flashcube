extern crate rs25;
extern crate vocs;
extern crate frontend as i73;
extern crate java_rand;
extern crate i73_biome;
extern crate i73_base;
extern crate i73_terrain;
extern crate i73_structure;
extern crate i73_decorator;
extern crate cgmath;
extern crate i73_shape;

use std::path::PathBuf;
use std::fs::File;
use std::cmp::min;

use i73::config::settings::customized::{Parts, BiomeSettings, Ocean, Structures, Decorators, VeinSettingsCentered, VeinSettings};
use i73_base::{Pass, Block};
use i73_terrain::overworld_173::{self, Settings};
use i73::config::biomes::{BiomesConfig, BiomeConfig, SurfaceConfig, RectConfig, FollowupConfig};
use i73_biome::Lookup;
use i73_base::matcher::BlockMatcher;

use cgmath::Vector3;

use vocs::indexed::ChunkIndexed;
use vocs::world::world::World;
use vocs::view::ColumnMut;
use vocs::position::{GlobalColumnPosition, GlobalChunkPosition};

use rs25::level::manager::{ColumnSnapshot, ChunkSnapshot};
use rs25::level::region::RegionWriter;
use rs25::level::anvil::ColumnRoot;
use std::collections::HashMap;
use i73_shape::volume::TriNoiseSettings;
use i73_shape::height::HeightSettings81;

fn main() {
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
		tri:            TriNoiseSettings { main_out_scale: 20.0, upper_out_scale: 512.0, lower_out_scale: 512.0, lower_scale: Vector3 { x: 684.412, y: 684.412, z: 684.412 }, upper_scale: Vector3 { x: 684.412, y: 684.412, z: 684.412 }, main_scale: Vector3 { x: 8.55515, y: 4.277575, z: 8.55515 }, y_size: 17 },
		height_stretch: 12.0,
		height:         HeightSettings81 { coord_scale: Vector3 { x: 200.0, y: 0.0, z: 200.0 }, out_scale: 8000.0, base: 8.5 },
		biome:          BiomeSettings { depth_weight: 1.0, depth_offset: 0.0, scale_weight: 1.0, scale_offset: 0.0, fixed: -1, biome_size: 4, river_size: 4 },
		ocean:          Ocean { top: 64, lava: false },
		structures:     Structures { caves: true, strongholds: true, villages: true, mineshafts: true, temples: true, ravines: true },
		decorators:     Decorators {
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
			lapis: VeinSettingsCentered { size: 7, count: 1, center_y: 16, spread: 16 }
		}
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
		
		if parts.ocean.lava { 11*16 } else { 9*16 }
	} else {
		0*16
	});
	
	settings.shape_blocks.ocean = sea_block;
	settings.paint_blocks.ocean = sea_block;

	// TODO: Structures and Decorators

	// TODO: Better JSON parser; uncommenting this adds 18 seconds to the compile time
	// let biomes_config = serde_json::from_reader::<File, BiomesConfig>(File::open(profile.join("biomes.json")).unwrap()).unwrap();
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

	let mut decorators: Vec<::i73_decorator::Dispatcher<i73_base::distribution::Chance<i73_base::distribution::Baseline>, i73_base::distribution::Chance<i73_base::distribution::Baseline>>> = Vec::new();

	/*for (name, decorator_set) in biomes_config.decorator_sets {
		println!("Configuring decorator set {}", name);

		for decorator_config in decorator_set {
			println!("Config: {:?}", decorator_config);

			decorators.push(decorator_config.into_dispatcher(&decorator_registry).unwrap());
		}
	}*/

	decorators.push (::i73_decorator::Dispatcher {
		decorator: Box::new(::i73_decorator::lake::LakeDecorator {
			blocks: ::i73_decorator::lake::LakeBlocks {
				is_liquid:  BlockMatcher::include([
					Block::from_anvil_id(8*16),
					Block::from_anvil_id(9*16),
					Block::from_anvil_id(10*16),
					Block::from_anvil_id(11*16)
				].iter()),
				is_solid:   BlockMatcher::exclude([
					Block::air(),
					Block::from_anvil_id(8*16),
					Block::from_anvil_id(9*16),
					Block::from_anvil_id(10*16),
					Block::from_anvil_id(11*16)
				].iter()), // TODO: All nonsolid blocks
				replaceable: BlockMatcher::none(), // TODO
				liquid:     Block::from_anvil_id(9*16),
				carve:      Block::air(),
				solidify:   None
			},
			settings: ::i73_decorator::lake::LakeSettings::default()
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
			base: ::i73_base::distribution::Baseline::Constant { value: 1 },
			chance: 4,
			ordering: ::i73_base::distribution::ChanceOrdering::AlwaysGeneratePayload
		}
	});

	decorators.push (::i73_decorator::Dispatcher {
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
	});

	//decorators.push (gravel_config.into_dispatcher(&decorator_registry).unwrap());

	decorators.push (::i73_decorator::Dispatcher {
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
				max: 90
			}),
			ordering: i73_base::distribution::ChanceOrdering::AlwaysGeneratePayload,
			chance: 1
		}
	});

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
	
	let (shape, paint) = overworld_173::passes(8399452073110208023, settings, Lookup::generate(&grid));
	
	let caves_generator = i73_structure::caves::CavesGenerator {
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
	let caves = i73_structure::StructureGenerateNearby::new(8399452073110208023, 8, caves_generator);
	
	/*let shape = nether_173::passes(-160654125608861039, &nether_173::default_tri_settings(), nether_173::ShapeBlocks::default(), 31);
	
	let default_grid = biome::default_grid();
	
	let mut fake_settings = Settings::default();
	fake_settings.biome_lookup = biome::Lookup::filled(default_grid.lookup(biome::climate::Climate::new(0.5, 0.0)));
	fake_settings.sea_coord = 31;
	fake_settings.beach = None;
	fake_settings.max_bedrock_height = None;
	
	let (_, paint) = overworld_173::passes(-160654125608861039, fake_settings);*/
	
	let mut world = World::<ChunkIndexed<Block>>::new();

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
				ChunkIndexed::<Block>::new(4, Block::air())
			];

			{
				let mut column: ColumnMut<Block> = ColumnMut::from_array(&mut column_chunks);

				shape.apply(&mut column, column_position);
				paint.apply(&mut column, column_position);
				caves.apply(&mut column, column_position);
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
	let coefficients = (
		((decoration_rng.next_i64() >> 1) << 1) + 1,
		((decoration_rng.next_i64() >> 1) << 1) + 1
	);

	for x in 0..31 {
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
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(dec_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Decoration done in {}us ({}us per column)", us, us / 1024);
	}

	use vocs::nibbles::{u4, ChunkNibbles, BulkNibbles};
	use vocs::mask::ChunkMask;
	use vocs::mask::LayerMask;
	use vocs::component::*;
	use vocs::view::{SplitDirectional, Directional};
	use rs25::dynamics::light::{SkyLightSources, Lighting, HeightMapBuilder};
	use rs25::dynamics::queue::Queue;
	use vocs::position::{Offset, dir};

	use vocs::world::shared::{NoPack, SharedWorld};

	let mut sky_light = SharedWorld::<NoPack<ChunkNibbles>>::new();
	let mut incomplete = World::<ChunkMask>::new();
	let mut heightmaps = ::std::collections::HashMap::<(i32, i32), Vec<u32>>::new(); // TODO: Better vocs integration.

	let mut lighting_info = HashMap::new()/*SparseStorage::<u4>::with_default(u4::new(15))*/;
	lighting_info.insert(Block::air(), u4::new(0));
	lighting_info.insert(Block::from_anvil_id( 8 * 16), u4::new(2));
	lighting_info.insert(Block::from_anvil_id( 9 * 16), u4::new(2));

	let empty_lighting = ChunkNibbles::default();

	let mut queue = Queue::default();

	println!("Performing initial sky lighting for region (0, 0)");
	let lighting_start = ::std::time::Instant::now();

	fn spill_out(chunk_position: GlobalChunkPosition, incomplete: &mut World<ChunkMask>, old_spills: vocs::view::Directional<LayerMask>) {
		if let Some(up) = chunk_position.plus_y() {
			if !old_spills[dir::Up].is_filled(false) {
				incomplete.get_or_create_mut(up).layer_zx_mut(0).combine(&old_spills[dir::Up]);
			}
		}

		if let Some(down) = chunk_position.minus_y() {
			if !old_spills[dir::Down].is_filled(false) {
				incomplete.get_or_create_mut(down).layer_zx_mut(15).combine(&old_spills[dir::Down]);
			}
		}

		if let Some(plus_x) = chunk_position.plus_x() {
			if !old_spills[dir::PlusX].is_filled(false) {
				incomplete.get_or_create_mut(plus_x).layer_zy_mut(0).combine(&old_spills[dir::PlusX]);
			}
		}

		if let Some(minus_x) = chunk_position.minus_x() {
			if !old_spills[dir::MinusX].is_filled(false) {
				incomplete.get_or_create_mut(minus_x).layer_zy_mut(15).combine(&old_spills[dir::MinusX]);
			}
		}

		if let Some(plus_z) = chunk_position.plus_z() {
			if !old_spills[dir::PlusZ].is_filled(false) {
				incomplete.get_or_create_mut(plus_z).layer_yx_mut(0).combine(&old_spills[dir::PlusZ]);
			}
		}

		if let Some(minus_z) = chunk_position.minus_z() {
			if !old_spills[dir::MinusZ].is_filled(false) {
				incomplete.get_or_create_mut(minus_z).layer_yx_mut(15).combine(&old_spills[dir::MinusZ]);
			}
		}
	}

	for x in 0..32 {
		println!("{}", x);
		for z in 0..32 {
			let column_position = GlobalColumnPosition::new(x, z);

			let mut mask = LayerMask::default();
			let mut heightmap = HeightMapBuilder::new();
			let mut heightmap_sections = [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None];

			for y in (0..16).rev() {
				let chunk_position = GlobalChunkPosition::from_column(column_position, y);

				let (blocks, palette) = world.get(chunk_position).unwrap().freeze();

				let mut opacity = BulkNibbles::new(palette.len());

				for (index, value) in palette.iter().enumerate() {
					opacity.set(index, value.and_then(|entry| lighting_info.get(&entry).map(|opacity| *opacity)).unwrap_or(u4::new(15)));
				}

				let sources = SkyLightSources::build(blocks, &opacity, mask);

				let mut light_data = ChunkNibbles::default();
				let neighbors = Directional::combine(SplitDirectional {
					minus_x: &empty_lighting,
					plus_x: &empty_lighting,
					minus_z: &empty_lighting,
					plus_z: &empty_lighting,
					down: &empty_lighting,
					up: &empty_lighting
				});

				let sources = {
					let mut light = Lighting::new(&mut light_data, neighbors, sources, opacity);

					light.initial(blocks, &mut queue);
					light.finish(blocks, &mut queue);

					light.decompose().1
				};

				heightmap_sections[y as usize] = Some(sources.clone());
				mask = heightmap.add(sources);

				let old_spills = queue.reset_spills();

				spill_out(chunk_position, &mut incomplete, old_spills);

				sky_light.set(chunk_position, NoPack(light_data));
			}

			let heightmap = heightmap.build();

			/*for (index, part) in heightmap_sections.iter().enumerate() {
				let part = part.as_ref().unwrap().clone();

				assert_eq!(SkyLightSources::slice(&heightmap, index as u8), part);
			}*/

			heightmaps.insert((x, z), heightmap.into_vec());
		}
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(lighting_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Initial sky lighting done in {}us ({}us per column)", us, us / 1024);
	}

	println!("Completing sky lighting for region (0, 0)");
	let complete_lighting_start = ::std::time::Instant::now();

	while incomplete.sectors().len() > 0 {
		let incomplete_front = ::std::mem::replace(&mut incomplete, World::new());

		for (sector_position, mut sector) in incomplete_front.into_sectors() {
			println!("Completing sector @ {} - {} queued", sector_position, sector.count_sectors());

			let block_sector = match world.get_sector(sector_position) {
				Some(sector) => sector,
				None => continue // No sense in lighting the void.
			};

			println!("(not skipped)");

			let light_sector = sky_light.get_or_create_sector_mut(sector_position);

			while let Some((position, incomplete)) = sector.pop_first() {
				use vocs::mask::Mask;
				println!("Completing chunk: {} / {} queued blocks", position, incomplete.count_ones());


				let (blocks, palette) = block_sector[position].as_ref().unwrap().freeze();

				let mut opacity = BulkNibbles::new(palette.len());

				for (index, value) in palette.iter().enumerate() {
					opacity.set(index, value.and_then(|entry| lighting_info.get(&entry).map(|opacity| *opacity)).unwrap_or(u4::new(15)));
				}

				let column_pos = GlobalColumnPosition::combine(sector_position, position.layer());
				let heightmap = heightmaps.get(&(column_pos.x(), column_pos.z())).unwrap();

				let sources = SkyLightSources::slice(&heightmap, position.y());

				// TODO: cross-sector lighting

				let mut central = light_sector.get_or_create(position);
				let locks = SplitDirectional {
					up: position.offset(dir::Up).map(|position| light_sector[position].read()),
					down: position.offset(dir::Down).map(|position| light_sector[position].read()),
					plus_x: position.offset(dir::PlusX).map(|position| light_sector[position].read()),
					minus_x: position.offset(dir::MinusX).map(|position| light_sector[position].read()),
					plus_z: position.offset(dir::PlusZ).map(|position| light_sector[position].read()),
					minus_z: position.offset(dir::MinusZ).map(|position| light_sector[position].read()),
				};

				let neighbors = SplitDirectional {
					up: locks.up.as_ref().and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0)).unwrap_or(&empty_lighting),
					down: locks.down.as_ref().and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0)).unwrap_or(&empty_lighting),
					plus_x: locks.plus_x.as_ref().and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0)).unwrap_or(&empty_lighting),
					minus_x: locks.minus_x.as_ref().and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0)).unwrap_or(&empty_lighting),
					plus_z: locks.plus_z.as_ref().and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0)).unwrap_or(&empty_lighting),
					minus_z: locks.minus_z.as_ref().and_then(|chunk| chunk.as_ref().map(|chunk| &chunk.0)).unwrap_or(&empty_lighting)
				};

				let mut light = Lighting::new(&mut central, Directional::combine(neighbors), sources, opacity);

				queue.reset_from_mask(incomplete);
				light.finish(blocks, &mut queue);

				// TODO: Queue handling
			}
		}
	}

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(complete_lighting_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Sky lighting completion done in {}us ({}us per column)", us, us / 1024);
	}

	println!("Writing region (0, 0)");
	let writing_start = ::std::time::Instant::now();

	// use rs25::level::manager::{Manager, RegionPool};
	// let pool = RegionPool::new(PathBuf::from("out/region/"), 512);
	// let mut manager = Manager::manage(pool);

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
				biomes: vec![0; 256],
				heightmap,
				entities: vec![],
				tile_entities: vec![],
				tile_ticks: vec![]
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
					sky_light: sky_light.0
				});
			};

			let root = ColumnRoot::from(snapshot.to_column(x as i32, z as i32).unwrap());

			writer.chunk(x as u8, z as u8, &root).unwrap();
		}
	}
	
	writer.finish().unwrap();

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(writing_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Writing done in {}us ({}us per column)", us, us / 1024);
		println!();
	}
}
