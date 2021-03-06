use crate::{Decorator, Result};
use i73_base::matcher::BlockMatcher;
use i73_base::block::Block;
use java_rand::Random;
use vocs::position::{dir, Offset, QuadPosition};
use vocs::view::QuadMut;

pub struct CactusDecorator {
	pub blocks: CactusBlocks,
	pub settings: CactusSettings,
}

impl Decorator for CactusDecorator {
	fn generate(
		&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition,
	) -> Result {
		if !self.blocks.replace.matches(quad.get(position)) {
			return Ok(());
		}

		let height = rng.next_u32_bound(self.settings.add_height + 1);
		let height = self.settings.base_height + rng.next_u32_bound(height + 1);

		let mut position = position;

		for _ in 0..height {
			position = match position.offset(dir::Up) {
				Some(position) => position,
				None => return Ok(()),
			};

			if self.blocks.check(quad, position) {
				quad.set_immediate(position, &self.blocks.block);
			}
		}

		Ok(())
	}
}

pub struct CactusBlocks {
	pub replace: BlockMatcher, // Air
	pub base: BlockMatcher,    // Cactus / Sand
	pub solid: BlockMatcher,   // any solid block
	pub block: Block,          // Cactus
}

impl CactusBlocks {
	pub fn check(&self, quad: &mut QuadMut<Block>, position: QuadPosition) -> bool {
		if !self.replace.matches(quad.get(position)) {
			return false;
		}

		if let Some(minus_x) = position.offset(dir::MinusX) {
			if self.solid.matches(quad.get(minus_x)) {
				return false;
			}
		}

		if let Some(plus_x) = position.offset(dir::PlusX) {
			if self.solid.matches(quad.get(plus_x)) {
				return false;
			}
		}

		if let Some(minus_z) = position.offset(dir::MinusZ) {
			if self.solid.matches(quad.get(minus_z)) {
				return false;
			}
		}

		if let Some(plus_z) = position.offset(dir::PlusZ) {
			if self.solid.matches(quad.get(plus_z)) {
				return false;
			}
		}

		let below = match position.offset(dir::Down) {
			Some(below) => below,
			None => return false,
		};

		self.base.matches(quad.get(below))
	}
}

pub struct CactusSettings {
	/// Base, minimum height of a cactus
	pub base_height: u32,
	/// Maximum height of a cactus when added to the base height.
	/// For example, with base=1 and add=2, the height of a cactus can be 1-3 blocks tall.
	pub add_height: u32,
}

impl Default for CactusSettings {
	fn default() -> Self {
		CactusSettings { base_height: 1, add_height: 2 }
	}
}

// Clump settings:
// iterations = 10
// horizontal_variation = 8
// vertical_variation = 4
