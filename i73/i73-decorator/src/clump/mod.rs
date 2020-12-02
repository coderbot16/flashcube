use crate::{Decorator, Result};
use i73_base::block::Block;
use java_rand::Random;
use vocs::position::{Offset, QuadPosition};
use vocs::view::QuadMut;

pub mod cactus;
pub mod plant;
pub mod sugar_cane;

/// Clumped generation. Places a number of objects with a varying distance from the center.
pub struct Clump<D>
where
	D: Decorator,
{
	pub iterations: u32,
	/// Horizontal variance. Must be 8 or below, or else spilling will occur.
	pub horizontal: u8,
	/// Vertical variance.
	pub vertical: u8,
	pub decorator: D,
}

impl<D> Decorator for Clump<D>
where
	D: Decorator,
{
	fn generate(
		&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition,
	) -> Result {
		for _ in 0..self.iterations {
			let offset = (
				rng.next_i32_bound(self.horizontal as i32)
					- rng.next_i32_bound(self.horizontal as i32),
				rng.next_i32_bound(self.vertical as i32) - rng.next_i32_bound(self.vertical as i32),
				rng.next_i32_bound(self.horizontal as i32)
					- rng.next_i32_bound(self.horizontal as i32),
			);

			if (position.y() as i32) + offset.1 < 0 {
				continue;
			}

			let at = match position.offset((offset.0 as i8, offset.1 as i8, offset.2 as i8)) {
				Some(at) => at,
				None => {
					panic!("out of bounds offsetting {:?} by {:?}", position, offset);
				}
			};

			self.decorator.generate(quad, rng, at)?;
		}

		Ok(())
	}
}

pub struct FlatClump<D>
where
	D: Decorator,
{
	pub iterations: u32,
	/// Horizontal variance. Must be 8 or below, or else spilling will occur.
	pub horizontal: u8,
	pub decorator: D,
}

impl<D> Decorator for FlatClump<D>
where
	D: Decorator,
{
	fn generate(
		&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition,
	) -> Result {
		for _ in 0..self.iterations {
			let offset = (
				rng.next_i32_bound(self.horizontal as i32)
					- rng.next_i32_bound(self.horizontal as i32),
				rng.next_i32_bound(self.horizontal as i32)
					- rng.next_i32_bound(self.horizontal as i32),
			);

			let at = position.offset((offset.0 as i8, 0, offset.1 as i8)).unwrap();

			self.decorator.generate(quad, rng, at)?;
		}

		Ok(())
	}
}
