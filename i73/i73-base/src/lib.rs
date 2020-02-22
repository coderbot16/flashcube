#[macro_use]
extern crate serde_derive;
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
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Block(u16);

impl Block {
	pub const fn from_anvil_id(id: u16) -> Self {
		Block(id)
	}

	pub const fn air() -> Self {
		Block(0)
	}
}

impl Into<u16> for Block {
	fn into(self) -> u16 {
		self.0
	}
}
