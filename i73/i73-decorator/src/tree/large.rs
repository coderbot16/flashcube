use super::TreeBlocks;
use crate::line::Line;
use crate::{Decorator, Result};
use i73_base::matcher::BlockMatcher;
use i73_base::Block;
use java_rand::Random;
use std::cmp::min;
use std::i32;
use vocs::position::{dir, Offset, QuadPosition};
use vocs::view::{QuadAssociation, QuadBlocks, QuadMut, QuadPalettes};

const TAU: f64 = 2.0 * 3.14159;

#[derive(Default)]
pub struct LargeTreeDecorator {
	blocks: TreeBlocks,
	settings: LargeTreeSettings,
}

impl LargeTreeDecorator {
	fn place_trunk(
		&self, position: QuadPosition, blocks: &mut QuadBlocks, palette: &QuadPalettes<Block>,
		log: &QuadAssociation, trunk_height: i32,
	) {
		let mut position = position;

		for _ in 0..trunk_height {
			if self.blocks.replace.matches(blocks.get(position, palette)) {
				blocks.set(position, log);
			}

			position = position.offset(dir::Up).unwrap();
		}
	}

	fn foliage_per_y(&self, height: f64) -> i32 {
		let height_factor = height / 13.0;

		min((self.settings.base_foliage_per_y + height_factor * height_factor) as i32, 1)
	}

	fn foliage(
		&self, trunk_height: i32, rng: &mut Random, spread: f64, y_offset: i32,
		origin: QuadPosition,
	) -> Foilage {
		let branch_factor = self.settings.branch_scale * spread * (rng.next_f32() as f64 + 0.328);
		let angle = (rng.next_f32() as f64) * TAU;

		let x = (branch_factor * angle.sin() + 0.5).floor() as i32;
		let z = (branch_factor * angle.cos() + 0.5).floor() as i32;

		let branch_length = ((x * x + z * z) as f64).sqrt();

		// Determine how low to place the branch start Y, controlled by branch_slope. Longer branches have lower starts on the trunk.
		let slope = (branch_length * self.settings.branch_slope) as i32;
		let branch_base = min(y_offset - slope, trunk_height);

		Foilage {
			base: origin.offset((x as i8, y_offset as i8, z as i8)).unwrap(),
			branch_y_offset: branch_base,
		}
	}
}

impl Decorator for LargeTreeDecorator {
	fn generate(
		&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition,
	) -> Result {
		let mut rng = Random::new(rng.next_u64());
		let height = self.settings.min_height + rng.next_i32_bound(self.settings.add_height + 1);
		let trunk_height =
			min((height as f64 * self.settings.trunk_height_scale) as i32, height - 1);

		/* TODO if tree.leaves_max_y > 128 {
			return Ok(());
		}*/

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
		let leaves = palette.reverse_lookup(&self.blocks.foliage).unwrap();

		Foilage {
			base: position.offset((0, (height - 4) as i8, 0)).unwrap(),
			branch_y_offset: trunk_height,
		}
		.place(&mut blocks, &leaves, &palette, &self.blocks.replace);

		let clusters = self.foliage_per_y(height as f64);

		for y_offset in ((height * 3) / 10..=height - 4).rev() {
			for _ in 0..clusters {
				let spread =
					0.5 * f64::sqrt((y_offset as f64) * (i32::abs(height - y_offset) as f64));

				let foliage = self.foliage(trunk_height, &mut rng, spread, y_offset, position);

				foliage.place(&mut blocks, &leaves, &palette, &self.blocks.replace);

				let tracer = Line {
					from: QuadPosition::new(
						position.x(),
						foliage.branch_y_offset as u8 + position.y(),
						position.z(),
					),
					to: foliage.base,
				}
				.trace();

				for limb in tracer {
					blocks.set(limb, &log);
				}
			}
		}

		self.place_trunk(position, &mut blocks, &palette, &log, height - 4 + 1);

		Ok(())
	}
}

/// A foliage cluster. "Balloon" oaks in Minecraft are simply a large tree generating a single foliage cluster at the top of the very short trunk.
#[derive(Debug)]
pub struct Foilage {
	/// Location of the leaf cluster, and the endpoint of the branch line. The Y is at the bottom of the cluster.
	base: QuadPosition,
	/// Y coordinate of the start of the branch line. The X and Z coordinate are always equal to the orgin of the tree.
	branch_y_offset: i32,
}

impl Foilage {
	fn place(
		&self, blocks: &mut QuadBlocks, foliage: &QuadAssociation, palette: &QuadPalettes<Block>,
		replace: &BlockMatcher,
	) {
		let mut position = self.base;

		Self::layer(1, position, blocks, foliage, palette, replace);

		for _ in 0..3 {
			position = position.offset(dir::Up).unwrap();
			Self::layer(2, position, blocks, foliage, palette, replace);
		}

		position = position.offset(dir::Up).unwrap();
		Self::layer(1, position, blocks, foliage, palette, replace);
	}

	fn layer(
		radius: i32, position: QuadPosition, blocks: &mut QuadBlocks, foliage: &QuadAssociation,
		palette: &QuadPalettes<Block>, replace: &BlockMatcher,
	) {
		for z_offset in -radius..=radius {
			for x_offset in -radius..=radius {
				if i32::abs(z_offset) == radius && i32::abs(x_offset) == radius {
					continue;
				}

				let position = match position.offset((x_offset as i8, 0, z_offset as i8)) {
					Some(position) => position,
					None => continue,
				};

				if replace.matches(blocks.get(position, palette)) {
					blocks.set(position, foliage);
				}
			}
		}
	}
}

#[derive(Debug)]
pub struct LargeTreeSettings {
	/// Makes the branches shorter or longer than the default.
	branch_scale: f64,
	/// For every 1 block the branch is long, this multiplier determines how many blocks it will go down on the trunk.
	branch_slope: f64,
	/// Default height of the leaves of the foliage clusters, from top to bottom.
	/// When added to the Y of the cluster, represents the coordinate of the top layer of the leaf cluster.
	foliage_height: i32,
	/// Factor in determining the amount of foliage clusters generated on each Y level of the big tree.
	foliage_density: f64,
	/// Added to the foliage_per_y value before conversion to i32.
	base_foliage_per_y: f64,
	/// How tall the trunk is in comparison to the total height. Should be 0.0 to 1.0.
	trunk_height_scale: f64,
	/// Minimum height of the tree.
	min_height: i32,
	/// Maximum height that can be added to the minimum. Max height of the tree = min_height + add_height.
	add_height: i32,
}

impl Default for LargeTreeSettings {
	fn default() -> Self {
		LargeTreeSettings {
			branch_scale: 1.0,
			branch_slope: 0.381,
			foliage_height: 4,
			foliage_density: 1.0,
			base_foliage_per_y: 1.382,
			trunk_height_scale: 0.618,
			min_height: 5,
			add_height: 11,
		}
	}
}
