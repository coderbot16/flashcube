extern crate deflate;
extern crate frontend as i73;
extern crate i73_base;
extern crate i73_biome;
extern crate i73_decorator;
extern crate i73_noise;
extern crate i73_structure;
extern crate i73_terrain;
extern crate java_rand;
extern crate nbt_turbo;
extern crate vocs;

use std::fs::File;

use i73_base::matcher::BlockMatcher;
use i73_base::{Block, Layer, Pass};
use i73_terrain::overworld::ocean::{OceanBlocks, OceanPass};
use i73_terrain::overworld_173::{self, Settings};

use vocs::indexed::ChunkIndexed;
use vocs::nibbles::u4;
use vocs::position::{
	GlobalChunkPosition, GlobalColumnPosition, GlobalSectorPosition, LayerPosition, QuadPosition,
};
use vocs::view::ColumnMut;
use vocs::world::world::World;

use i73_decorator::tree::{LargeTreeDecorator, NormalTreeDecorator};
use i73_decorator::Decorator;
use i73_noise::sample::Sample;
use std::collections::HashMap;
use vocs::nibbles::ChunkNibbles;
use vocs::world::shared::{NoPack, SharedWorld};
use vocs::position::{dir, Offset};

fn main() {
	let main_start = ::std::time::Instant::now();

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

	let ocean = OceanPass {
		blocks: OceanBlocks::default(),
		sea_top: 64,
	};

	let biome_lookup = i73::generate_biome_lookup();
	let (climates, shape, paint) =
		overworld_173::passes(8399452073110208023, Settings::default(), biome_lookup);

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

	println!("Computing heightmaps");
	let heightmaps_start = std::time::Instant::now();

	let mut lighting_info = HashMap::new();

	lighting_info.insert(Block::air(), u4::new(0));
	lighting_info.insert(Block::from_anvil_id(8 * 16), u4::new(2));
	lighting_info.insert(Block::from_anvil_id(9 * 16), u4::new(2));
	lighting_info.insert(Block::from_anvil_id(18 * 16), u4::new(1));

	let predicate = |block| {
		lighting_info.get(block).copied().unwrap_or(u4::new(15)) != u4::new(0)
	};

	let mut heightmaps = lumis::compute_world_heightmaps(&world, &predicate);

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(heightmaps_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Heightmap computation done in {}us ({}us per column)", us, us / 1024);
	}

	println!("Performing sky lighting");
	let lighting_start = std::time::Instant::now();

	let opacities = |block| lighting_info.get(block).copied().unwrap_or(u4::new(15));

	// Also logs timing messages
	let mut sky_light = lumis::compute_world_skylight(&world, &heightmaps, &opacities, &lumis::PrintTraces);

	{
		let end = ::std::time::Instant::now();
		let time = end.duration_since(lighting_start);

		let secs = time.as_secs();
		let us = (secs * 1000000) + ((time.subsec_nanos() / 1000) as u64);

		println!("Sky lighting done in {}us ({}us per column)", us, us / 1024);
	}

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

/*fn write_classicworld(world: &World<ChunkIndexed<Block>>) {
	use vocs::position::ChunkPosition;

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
	use deflate::Compression;
	use std::io::Write;

	let file = File::create("out/classic/i73.cw").unwrap();
	let mut gzip = GzEncoder::new(file, Compression::Fast);
	gzip.write_all(&buffer).unwrap();
	gzip.finish().unwrap();
}*/

fn write_region(
	world: &World<ChunkIndexed<Block>>, sky_light: &mut SharedWorld<NoPack<ChunkNibbles>>,
	heightmaps: &mut HashMap<GlobalSectorPosition, vocs::unpacked::Layer<lumis::heightmap::ColumnHeightMap>>,
	world_biomes: &mut HashMap<(i32, i32), Vec<u8>>,
) {
	use mca::{AnvilBlocks, Column, ColumnRoot, Section, SectionRef};
	use region::{RegionWriter, ZlibOutput};

	let region_file = File::create("out/region/r.0.0.mca").unwrap();
	let mut writer = RegionWriter::start(region_file).unwrap();

	// Split up the heightmaps into the format expected by the rest of i73
	let mut individual_heightmaps: HashMap<GlobalColumnPosition, lumis::heightmap::ColumnHeightMap> = HashMap::new();

	heightmaps.drain().for_each(|(position, sector_heightmaps)| {
		for (index, heightmap) in sector_heightmaps.into_inner().into_vec().into_iter().enumerate() {
			let layer = LayerPosition::from_zx(index as u8);
			let column_position = GlobalColumnPosition::combine(position, layer);
	
			individual_heightmaps.insert(column_position, heightmap);
		}
	});

	for z in 0..32 {
		println!("{}", z);
		for x in 0..32 {
			let column_position = GlobalColumnPosition::new(x, z);

			let heightmap: Box<[u32]> = individual_heightmaps.remove(&column_position).unwrap().into_inner();
			let biomes = world_biomes.remove(&(x, z)).unwrap();

			let mut sections = Vec::new();

			for y in 0..16 {
				let chunk_position = GlobalChunkPosition::from_column(column_position, y);

				let chunk = world.get(chunk_position).unwrap();

				let anvil_blocks = AnvilBlocks::from_paletted(&chunk, &|&id| id.into());

				/*if chunk.anvil_empty() {
					continue;
				}*/

				let sky_light = sky_light.remove(chunk_position).unwrap().0/*_or_else(ChunkNibbles::default)*/;

				sections.push(Section {
					y: y as i8,
					blocks: anvil_blocks.blocks,
					add: anvil_blocks.add,
					data: anvil_blocks.data,
					block_light: ChunkNibbles::default(),
					sky_light
				});
			}
			
			let section_refs: Vec<SectionRef> = sections.iter().map(Section::to_ref).collect();
			
			let column = Column {
				x: x as i32,
				z: z as i32,
				last_update: 0,
				light_populated: true,
				terrain_populated: true,
				v: Some(1),
				inhabited_time: 0,
				biomes: &biomes,
				heightmap: &heightmap,
				sections: &section_refs,
				tile_ticks: &[]
			};

			let root = ColumnRoot {
				version: Some(0),
				column: column
			};

			let mut output = ZlibOutput::new();
			root.write(&mut output);
			writer.column(x as u8, z as u8, &output.finish()).unwrap();
		}
	}

	writer.finish().unwrap();
}
