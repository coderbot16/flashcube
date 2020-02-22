#[macro_use]
extern crate serde_derive;
extern crate java_rand;
extern crate vocs;
extern crate i73_base;
extern crate i73_trig;

use java_rand::Random;
use vocs::view::QuadMut;
use vocs::position::{ColumnPosition, QuadPosition};
use i73_base::distribution::Distribution;
use i73_base::Block;

pub mod dungeon;
pub mod vein;
pub mod clump;
// TODO: pub mod large_tree;
pub mod lake;
pub mod tree;
pub mod exposed;

// TODO: MultiDispatcher

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Spilled(pub QuadPosition);
pub type Result = ::std::result::Result<(), Spilled>;

pub struct Dispatcher<H, R> where H: Distribution, R: Distribution {
	pub height_distribution: H,
	pub rarity: R,
	pub decorator: Box<dyn Decorator>
}

impl<H, R> Dispatcher<H, R> where H: Distribution, R: Distribution {
	pub fn generate(&self, quad: &mut QuadMut<Block>, rng: &mut Random) -> Result {
		for _ in 0..self.rarity.next(rng) {
			let at = ColumnPosition::new(
				rng.next_u32_bound(16) as u8,
				self.height_distribution.next(rng) as u8,
				rng.next_u32_bound(16) as u8
			);
			
			self.decorator.generate(quad, rng, QuadPosition::from_centered(at))?;
		}
		
		Ok(())
	}
}

pub trait Decorator {
	fn generate(&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition) -> Result;
}