use crate::{Decorator, Result};
use i73_base::matcher::BlockMatcher;
use i73_base::block::Block;
use java_rand::Random;
use vocs::position::{dir, Offset, QuadPosition};
use vocs::view::QuadMut;

// Pumpkin: On grass, replacing air or {material:ground_cover}

pub struct PlantDecorator {
	pub block: Block,
	pub base: BlockMatcher,
	pub replace: BlockMatcher,
}

impl Decorator for PlantDecorator {
	fn generate(
		&self, quad: &mut QuadMut<Block>, _: &mut Random, position: QuadPosition,
	) -> Result {
		// TODO: Check if the block is above the heightmap (how?)

		if !self.replace.matches(quad.get(position)) {
			return Ok(());
		}

		match position.offset(dir::Down) {
			Some(below) => {
				if !self.base.matches(quad.get(below)) {
					return Ok(());
				}
			}
			None => return Ok(()),
		}

		quad.set_immediate(position, &self.block);

		Ok(())
	}
}
