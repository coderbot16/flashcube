extern crate fxhash;
extern crate java_rand;
extern crate vocs;

pub mod distribution;
pub mod matcher;
pub mod math;

mod layer;
pub use layer::Layer;

use vocs::position::GlobalColumnPosition;
use vocs::view::ColumnMut;

pub trait Pass<C: Copy> {
	fn apply(&self, target: &mut ColumnMut<Block>, climate: &Layer<C>, chunk: GlobalColumnPosition);
}

/// ID of a block.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Block(u16);

impl Block {
	pub const AIR: Block = Block(0);
	pub const STONE: Block = Block(1 * 16);
	pub const GRASS: Block = Block(2 * 16);
	pub const DIRT: Block = Block(3 * 16);
	pub const BEDROCK: Block = Block(7 * 16);
	pub const FLOWING_WATER: Block = Block(8 * 16);
	pub const STILL_WATER: Block = Block(9 * 16);
	pub const FLOWING_LAVA: Block = Block(10 * 16);
	pub const STILL_LAVA: Block = Block(11 * 16);
	pub const SAND: Block = Block(12 * 16);
	pub const GRAVEL: Block = Block(13 * 16);
	pub const OAK_LOG: Block = Block(17 * 16);
	pub const OAK_LEAVES: Block = Block(18 * 16);
	pub const SANDSTONE: Block = Block(24 * 16);
	pub const ICE: Block = Block(79 * 16);
	pub const NETHERRACK: Block = Block(87 * 16);

	pub const fn from_anvil_id(id: u16) -> Self {
		Block(id)
	}
}

impl Into<u16> for Block {
	fn into(self) -> u16 {
		self.0
	}
}
