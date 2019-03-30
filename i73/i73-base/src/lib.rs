#[macro_use]
extern crate serde_derive;
extern crate vocs;
extern crate java_rand;

pub mod distribution;
pub mod matcher;

use vocs::indexed::Target;
use vocs::view::ColumnMut;
use vocs::position::GlobalColumnPosition;

pub trait Pass<B> where B: Target {
	fn apply(&self, target: &mut ColumnMut<B>, chunk: GlobalColumnPosition);
}