#[macro_use]
extern crate serde_derive;
extern crate vocs;
extern crate java_rand;

pub mod distribution;
pub mod matcher;
use vocs::view::ColumnMut;
use vocs::position::GlobalColumnPosition;

pub trait Pass {
	fn apply(&self, target: &mut ColumnMut<Block>, chunk: GlobalColumnPosition);
}

/// ID of a block.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Block(u16);

impl Block {
	pub fn from_anvil_id(id: u16) -> Self {
		Block(id)
	}

	pub fn air() -> Self {
		Block(0)
	}
}

impl Into<u16> for Block {
	fn into(self) -> u16 {
		self.0
	}
}