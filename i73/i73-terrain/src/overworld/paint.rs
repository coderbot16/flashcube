use java_rand::Random;
use cgmath::{Point2, Vector2, Vector3};
use i73_noise::octaves::PerlinOctaves;
use i73_noise::sample::Sample;
use i73_biome::climate::{ClimateSettings, ClimateSource, Climate};
use i73_biome::{Lookup, Surface};
use i73_shape::height::{HeightSettings, HeightSource};
use i73_shape::volume::{TriNoiseSettings, TriNoiseSource, ShapeSettings};
use i73_base::{Pass, Layer};
use vocs::position::{ColumnPosition, LayerPosition, GlobalColumnPosition};
use vocs::view::{ColumnMut, ColumnBlocks, ColumnPalettes, ColumnAssociation};
use i73_base::matcher::BlockMatcher;
use i73_base::Block;

use overworld::shape::{ShapeBlocks, ShapePass};

pub struct Settings {
	pub shape_blocks: ShapeBlocks,
	pub paint_blocks: PaintBlocks,
	pub tri:          TriNoiseSettings,
	pub height:       HeightSettings,
	pub field:        ShapeSettings,
	pub sea_coord:    u8,
	pub beach:        Option<(u8, u8)>,
	pub max_bedrock_height: Option<u8>,
	pub climate:      ClimateSettings
}

impl Default for Settings {
	fn default() -> Self {
		Settings {
			shape_blocks: ShapeBlocks::default(),
			paint_blocks: PaintBlocks::default(),
			tri:          TriNoiseSettings::default(),
			height:       HeightSettings::default(),
			field:        ShapeSettings::default(),
			sea_coord:    63,
			beach:        Some((59, 65)),
			max_bedrock_height: Some(5),
			climate:      ClimateSettings::default()
		}
	}
}

pub fn passes(seed: u64, settings: Settings, biome_lookup: Lookup) -> (ClimateSource, ShapePass, PaintPass) {
	let mut rng = Random::new(seed);
	
	let tri = TriNoiseSource::new(&mut rng, &settings.tri);
	
	// TODO: The PerlinOctaves implementation currently does not support noise on arbitrary Y coordinates.
	// Oddly, this "feature" is what causes the sharp walls in beach/biome surfaces.
	// It is a mystery why the feature exists in the first place.
	
	let sand      = PerlinOctaves::new(&mut rng.clone(), 4, Vector3::new(1.0 / 32.0, 1.0 / 32.0,        1.0)); // Vertical,   Z =   0.0
	let gravel    = PerlinOctaves::new(&mut rng,                        4, Vector3::new(1.0 / 32.0,        1.0, 1.0 / 32.0)); // Horizontal
	let thickness = PerlinOctaves::new(&mut rng,                        4, Vector3::new(1.0 / 16.0, 1.0 / 16.0, 1.0 / 16.0)); // Vertical,   Z =   0.0
	
	let height  = HeightSource::new(&mut rng, &settings.height);
	let field   = settings.field;

	(
		ClimateSource::new(seed, settings.climate),
		ShapePass {
			blocks: settings.shape_blocks, 
			tri, 
			height, 
			field
		},
		PaintPass {
			lookup: biome_lookup,
			blocks: settings.paint_blocks, 
			sand, 
			gravel, 
			thickness, 
			sea_coord: settings.sea_coord, 
			beach: settings.beach,
			max_bedrock_height: settings.max_bedrock_height 
		}
	)
}

pub struct PaintBlocks {
	pub reset:     BlockMatcher,
	pub ignore:    BlockMatcher,
	pub air:       Block,
	pub stone:     Block,
	pub gravel:    Block,
	pub sand:      Block,
	pub sandstone: Block,
	pub bedrock:   Block
}

impl Default for PaintBlocks {
	fn default() -> Self {
		PaintBlocks {
			reset:     BlockMatcher::is(Block::air()),
			ignore:    BlockMatcher::is_not(Block::from_anvil_id(1 * 16)),
			air:       Block::air(),
			stone:     Block::from_anvil_id( 1 * 16),
			gravel:    Block::from_anvil_id(13 * 16),
			sand:      Block::from_anvil_id(12 * 16),
			sandstone: Block::from_anvil_id(24 * 16),
			bedrock:   Block::from_anvil_id( 7 * 16)
		}
	}
}

struct SurfaceAssociations {
	pub top:  ColumnAssociation,
	pub fill: ColumnAssociation,
	pub chain: Vec<FollowupAssociation>
}

impl SurfaceAssociations {
	fn lookup(surface: &Surface, palette: &ColumnPalettes<Block>) -> Self {
		let mut chain = Vec::new();
		
		for followup in &surface.chain {
			chain.push(
				FollowupAssociation {
					block:     palette.reverse_lookup(&followup.block).unwrap(),
					max_depth: followup.max_depth
				} 
			)
		}
		
		SurfaceAssociations {
			top:   palette.reverse_lookup(&surface.top).unwrap(),
			fill:  palette.reverse_lookup(&surface.fill).unwrap(),
			chain
		}
	}
}

struct FollowupAssociation {
	pub block:     ColumnAssociation,
	pub max_depth: u32
}

pub struct PaintPass {
	pub lookup:    Lookup,
	pub blocks:    PaintBlocks,
	pub sand:      PerlinOctaves,
	pub gravel:    PerlinOctaves,
	pub thickness: PerlinOctaves,
	pub sea_coord: u8,
	pub beach:     Option<(u8, u8)>,
	pub max_bedrock_height: Option<u8>
}

impl PaintPass {
	pub fn biome_lookup(&self) -> &Lookup {
		&self.lookup
	}

	fn paint_stack(&self, rng: &mut Random, blocks: &mut ColumnBlocks, palette: &ColumnPalettes<Block>, bedrock: &ColumnAssociation, layer: LayerPosition, surface: &SurfaceAssociations, beach: &SurfaceAssociations, basin: &SurfaceAssociations, thickness: i32, max_y: u8) {
		let reset_remaining = match thickness {
			-1          => None,
			x if x <= 0 => Some(0),
			thickness   => Some(thickness as u32)
		};
		
		let mut remaining = None;
		let mut followup_index: Option<usize> = None;
		
		let mut current_surface = if thickness <= 0 {basin} else {surface};
		
		for y in (0..max_y).rev() {
			let position = ColumnPosition::from_layer(y, layer);
			
			if let Some(chance) = self.max_bedrock_height {
				if (y as u32) <= rng.next_u32_bound(chance as u32) {
					blocks.set(position, bedrock);
					continue;
				}
			}
			
			let existing_block = blocks.get(position, &palette);

			if self.blocks.reset.matches(existing_block) {
				if y > self.sea_coord {
					remaining = None;
				}

				continue;
			}
			
			match remaining {
				Some(0) => (),
				Some(ref mut remaining) => {
					let block = match followup_index {
						Some(index) => &current_surface.chain[index].block,
						None =>        &current_surface.fill
					};
					
					blocks.set(position, block);
					
					*remaining -= 1;
					if *remaining == 0 {
						// TODO: Don't increment the index if it is already out of bounds.
						let new_index = followup_index.map(|index| index + 1).unwrap_or(0);
						
						if new_index < current_surface.chain.len() {
							*remaining = rng.next_u32_bound(current_surface.chain[new_index].max_depth + 1)
						}
						
						followup_index = Some(new_index);
					}
				},
				None => {
					if thickness <= 0 {
						current_surface = basin;
					} else if let Some(beach_range) = self.beach {
						if y >= beach_range.0 && y <= beach_range.1 {
							current_surface = beach;
						}
					}
			
					blocks.set(position, if y >= self.sea_coord {&current_surface.top} else {&current_surface.fill});
			
					remaining = reset_remaining;
					followup_index = None;
				}
			}
		}
	}
}

impl Pass<Climate> for PaintPass {
	fn apply(&self, target: &mut ColumnMut<Block>, climates: &Layer<Climate>, chunk: GlobalColumnPosition) {
		let mut max_y = 0;
		for (index, chunk) in target.0.iter().take(8).enumerate() {
			if !chunk.is_filled_heuristic(&self.blocks.air) {
				max_y = (index as u8 + 1) * 16;
			}
		}

		let block = ((chunk.x() * 16) as f64, (chunk.z() * 16) as f64);
		let seed = (chunk.x() as i64).wrapping_mul(341873128712).wrapping_add((chunk.z() as i64).wrapping_mul(132897987541));
		let mut rng = Random::new(seed as u64);

		let biome_layer = self.lookup.climates_to_biomes(&climates);
		let (biomes, biome_palette) = biome_layer.freeze();
		
		let      sand_vertical = self.     sand.vertical_ref(block.1, 16);
		let thickness_vertical = self.thickness.vertical_ref(block.1, 16);
		
		let   vertical_offset = Vector3::new(block.0 as f64, block.1 as f64, 0.0);
		let horizontal_offset = Point2::new(block.0 as f64, block.1 as f64);
		
		target.ensure_available(self.blocks.air.clone());
		target.ensure_available(self.blocks.stone.clone());
		target.ensure_available(self.blocks.gravel.clone());
		target.ensure_available(self.blocks.sand.clone());
		target.ensure_available(self.blocks.sandstone.clone());
		target.ensure_available(self.blocks.bedrock.clone());
		
		for surface in biome_palette.iter().filter_map(Option::as_ref).map(|biome| &biome.surface) {
			target.ensure_available(surface.top.clone());
			target.ensure_available(surface.fill.clone());
				
			for followup in &surface.chain {
				target.ensure_available(followup.block.clone());
			}
		}
		
		let (mut blocks, palette) = target.freeze_palette();
		
		let mut surfaces = Vec::new();
		
		for entry in biome_palette {
			surfaces.push(
				entry.as_ref().map(|biome| SurfaceAssociations::lookup(&biome.surface, &palette))
			);
		}
		
		let bedrock = palette.reverse_lookup(&self.blocks.bedrock).unwrap();
		
		let gravel_beach = SurfaceAssociations {
			top:   palette.reverse_lookup(&self.blocks.air).unwrap(),
			fill:  palette.reverse_lookup(&self.blocks.gravel).unwrap(),
			chain: vec![]
		};
		
		let sand_beach   = SurfaceAssociations {
			top:   palette.reverse_lookup(&self.blocks.sand).unwrap(),
			fill:  palette.reverse_lookup(&self.blocks.sand).unwrap(),
			chain: vec![
				FollowupAssociation {
					block:     palette.reverse_lookup(&self.blocks.sandstone).unwrap(),
					max_depth: 3
				}
			]
		};
		
		let basin        = SurfaceAssociations {
			top:   palette.reverse_lookup(&self.blocks.air).unwrap(),
			fill:  palette.reverse_lookup(&self.blocks.stone).unwrap(),
			chain: vec![]
		};

		for position in LayerPosition::enumerate() {
			// TODO: BeachSelector

			let (x, z) = (position.x() as f64, position.z() as f64);

			let (sand_variation, gravel_variation, thickness_variation) = (rng.next_f64() * 0.2, rng.next_f64() * 0.2, rng.next_f64() * 0.25);

			let   sand    =       sand_vertical.generate_override(  vertical_offset + Vector3::new(x, z, 0.0), position.z() as usize) +   sand_variation > 0.0;
			let gravel    =         self.gravel.sample           (horizontal_offset + Vector2::new(x, z     )            ) + gravel_variation > 3.0;
			let thickness = (thickness_vertical.generate_override(  vertical_offset + Vector3::new(x, z, 0.0), position.z() as usize) / 3.0 + 3.0 + thickness_variation) as i32;

			let surface   = surfaces[biomes.get(position) as usize].as_ref().unwrap();

			let beach = if sand {
				&sand_beach
			} else if gravel {
				&gravel_beach
			} else {
				surface
			};

			self.paint_stack(&mut rng, &mut blocks, &palette, &bedrock, position, surface, beach, &basin, thickness, max_y);
		}
	}
}