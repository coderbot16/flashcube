use crate::{Decorator, Result};
use i73_base::matcher::BlockMatcher;
use i73_base::block::Block;
use java_rand::Random;
use vocs::position::{dir, Offset, QuadPosition};
use vocs::view::QuadMut;

pub struct SugarCaneDecorator {
	pub block: Block,
	pub base: BlockMatcher,
	pub liquid: BlockMatcher,
	pub replace: BlockMatcher,
	pub base_height: u32,
	pub add_height: u32,
}

impl Decorator for SugarCaneDecorator {
	fn generate(
		&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition,
	) -> Result {
		if !self.replace.matches(quad.get(position)) {
			return Ok(());
		}

		let below = match position.offset(dir::Down) {
			Some(below) => below,
			None => return Ok(()),
		};

		if *quad.get(below) != self.block {
			if !self.base.matches(quad.get(below)) {
				return Ok(());
			}

			let mut valid = false;

			if let Some(minus_x) = below.offset(dir::MinusX) {
				if self.liquid.matches(quad.get(minus_x)) {
					valid = true;
				}
			}

			if let Some(plus_x) = below.offset(dir::PlusX) {
				if self.liquid.matches(quad.get(plus_x)) {
					valid = true;
				}
			}

			if let Some(minus_z) = below.offset(dir::MinusZ) {
				if self.liquid.matches(quad.get(minus_z)) {
					valid = true;
				}
			}

			if let Some(plus_z) = below.offset(dir::PlusZ) {
				if self.liquid.matches(quad.get(plus_z)) {
					valid = true;
				}
			}

			if !valid {
				return Ok(());
			}
		}

		let height = rng.next_u32_bound(self.add_height + 1);
		let height = self.base_height + rng.next_u32_bound(height + 1);

		let mut position = position;

		for _ in 0..height {
			if !self.replace.matches(quad.get(position)) {
				return Ok(());
			}

			quad.set_immediate(position, &self.block);

			if let Some(at) = position.offset(dir::Up) {
				position = at;
			} else {
				return Ok(());
			}
		}

		Ok(())
	}
}
