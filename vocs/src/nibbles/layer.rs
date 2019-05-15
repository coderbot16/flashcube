use std::fmt::{Debug, Formatter, Result};
use crate::position::LayerPosition;
use super::{u4, u4x2, nibble_index};
use crate::component::LayerStorage;

/// A 16x16 collection of nibbles (`u4`s).
/// Indexed with LayerPosition.
pub struct LayerNibbles([u4x2; 128]);
impl LayerNibbles {
	/// Sets a value, without clearing what was there previously.
	/// This uses the `u4x2::replace_or` function internally, and shares the same semantics.
	/// This can be used as an optimization to avoid clearing an already cleared value when operating
	/// on a fresh buffer.
	pub fn set_uncleared(&mut self, at: LayerPosition, value: u4) {
		let (index, shift) = nibble_index(at.zx() as usize);

		self.0[index] = self.0[index].replace_or(shift, value);
	}

	/// Returns a reference to the raw array of `u4x2`s.
	/// `a` is the even element, `b` is the odd element.
	/// For example:
	/// (x:0,z:0) is index 0, element `a`.
	/// (x:15,z:15) is index 127, element `b`.
	pub fn raw(&self) -> &[u4x2; 128] {
		&self.0
	}
}

impl LayerStorage<u4> for LayerNibbles {
	fn get(&self, at: LayerPosition) -> u4 {
		let (index, shift) = nibble_index(at.zx() as usize);

		self.0[index].extract(shift)
	}

	fn is_filled(&self, value: u4) -> bool {
		let fill = u4x2::splat(value);

		for &term in self.0.iter() {
			if term != fill {
				return false;
			}
		}

		true
	}

	fn set(&mut self, at: LayerPosition, value: u4) {
		let (index, shift) = nibble_index(at.zx() as usize);

		self.0[index] = self.0[index].replace(shift, value);
	}

	fn fill(&mut self, value: u4) {
		let fill = u4x2::splat(value);

		for term in &mut self.0 as &mut [u4x2] {
			*term = fill;
		}
	}
}

impl Default for LayerNibbles {
	fn default() -> Self {
		LayerNibbles([u4x2::default(); 128])
	}
}

impl Debug for LayerNibbles {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{:?}", &self.0[..])
	}
}

impl Clone for LayerNibbles {
	fn clone(&self) -> Self {
		let mut other = [u4x2::default(); 128];

		other.copy_from_slice(&self.0[..]);

		LayerNibbles(other)
	}
}

impl PartialEq for LayerNibbles {
	fn eq(&self, other: &LayerNibbles) -> bool {
		&self.0[..] == &other.0[..]
	}
}

impl Eq for LayerNibbles {}