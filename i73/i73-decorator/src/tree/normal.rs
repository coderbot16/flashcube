use super::{FoliageLayer, TreeBlocks};
use crate::{Decorator, Result};
use i73_base::Block;
use java_rand::Random;
use vocs::position::{dir, Offset, QuadPosition};
use vocs::view::QuadMut;

#[derive(Default)]
pub struct NormalTreeDecorator {
	blocks: TreeBlocks,
	settings: TreeSettings,
}

impl Decorator for NormalTreeDecorator {
	fn generate(
		&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition,
	) -> Result {
		let tree = self.settings.tree(rng, position.y());

		if tree.leaves_max_y > 128 {
			return Ok(());
		}

		let below = match position.offset(dir::Down) {
			Some(below) => below,
			None => return Ok(()),
		};

		if !self.blocks.soil.matches(quad.get(below)) {
			return Ok(());
		}

		// TODO: Check bounding box

		quad.set_immediate(below, &self.blocks.new_soil);

		quad.ensure_available(self.blocks.log.clone());
		quad.ensure_available(self.blocks.foliage.clone());

		let (mut blocks, palette) = quad.freeze_palette();

		let log = palette.reverse_lookup(&self.blocks.log).unwrap();
		let foliage = palette.reverse_lookup(&self.blocks.foliage).unwrap();

		for y in tree.leaves_min_y..=tree.leaves_max_y {
			let radius = tree.foliage_radius(y);

			let position = QuadPosition::new(position.x(), y as u8, position.z());
			let layer = FoliageLayer { position, radius: radius as u8 };

			layer.place(&mut blocks, &foliage, &palette, &self.blocks.replace);
			layer.place_corners(&mut blocks, &foliage, &palette, &self.blocks.replace, |y| {
				rng.next_u32_bound(self.settings.foliage_corner_chance) != 0
					&& y < tree.trunk_top as u8
			});
		}

		for y in position.y()..(tree.trunk_top as u8) {
			let position = QuadPosition::new(position.x(), y, position.z());

			if self.blocks.replace.matches(blocks.get(position, &palette)) {
				blocks.set(position, &log);
			}
		}

		Ok(())
	}
}

struct TreeSettings {
	min_trunk_height: u32,
	add_trunk_height: u32,
	foliage_layers_on_trunk: u32,
	foliage_layers_off_trunk: u32,
	foliage_slope: u32,
	foliage_radius_base: u32,
	foliage_corner_chance: u32,
}

impl TreeSettings {
	fn tree(&self, rng: &mut Random, origin_y: u8) -> Tree {
		let trunk_height = self.min_trunk_height + rng.next_u32_bound(self.add_trunk_height + 1);
		let trunk_top = (origin_y as u32) + trunk_height;

		Tree {
			// full_height: trunk_height + self.foliage_layers_off_trunk,
			// trunk_height,
			trunk_top,
			leaves_min_y: trunk_top - self.foliage_layers_on_trunk,
			leaves_max_y: trunk_top + self.foliage_layers_off_trunk,
			leaves_slope: self.foliage_slope,
			leaves_radius_base: self.foliage_radius_base,
		}
	}
}

impl Default for TreeSettings {
	fn default() -> Self {
		TreeSettings {
			min_trunk_height: 4,
			add_trunk_height: 2,
			foliage_layers_on_trunk: 3,
			foliage_layers_off_trunk: 1,
			foliage_slope: 2,
			foliage_radius_base: 1,
			foliage_corner_chance: 2,
		}
	}
}

struct Tree {
	// Trunk Height + number of foliage layers above the trunk
	// full_height: u32,
	// Height of the trunk. Can be considered the length of the line that defines the trunk.
	// trunk_height: u32,
	/// Coordinates of the block above the last block of the trunk.
	trunk_top: u32,
	/// Minimum Y value for foliage layers (Inclusive).
	leaves_min_y: u32,
	/// Maximum Y value for foliage layers (Exclusive).
	leaves_max_y: u32,
	/// Slope value of the radius for each layer. Flattens or widens the tree.
	leaves_slope: u32,
	/// Base value for the radius.
	leaves_radius_base: u32,
}

impl Tree {
	/// Radius of the foliage at a given location. 0 is just the trunk.
	fn foliage_radius(&self, y: u32) -> u32 {
		(self.leaves_radius_base + self.trunk_top + 1 - y) / self.leaves_slope
	}

	// Radius of the bounding box for the foliage at a given level. 0 for just checking the trunk.
	/*fn bounding_radius(&self, y: u32) -> u32 {
		if y == (self.orgin.y() as u32) {
			0
		} else if y > self.trunk_top {
			2
		} else {
			1
		}
	}*/
}
