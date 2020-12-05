extern crate fxhash;
extern crate java_rand;
extern crate vocs;

pub mod block;
pub mod distribution;
pub mod matcher;
pub mod math;

use block::Block;
use vocs::position::GlobalColumnPosition;
use vocs::view::ColumnMut;
use vocs::unpacked::Layer;

pub trait Pass<C: Copy> {
	fn apply(&self, target: &mut ColumnMut<Block>, climate: &Layer<C>, chunk: GlobalColumnPosition);
}
