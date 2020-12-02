extern crate fxhash;
extern crate java_rand;
extern crate vocs;

pub mod block;
pub mod distribution;
pub mod matcher;
pub mod math;

mod layer;
pub use layer::Layer;

use block::Block;
use vocs::position::GlobalColumnPosition;
use vocs::view::ColumnMut;

pub trait Pass<C: Copy> {
	fn apply(&self, target: &mut ColumnMut<Block>, climate: &Layer<C>, chunk: GlobalColumnPosition);
}
