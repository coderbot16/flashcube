use std::fmt::{Debug, Formatter, Result};
use crate::position::CubePosition;
use super::{u4, nibble_index};
use crate::component::CubeStorage;

/// A 16x16 collection of nibbles (`u4`s).
/// Indexed with CubePosition.
#[derive(Eq, PartialEq)]
pub struct NibbleCube(Box<[u8; 2048]>);
impl NibbleCube {
	/// Creates a `NibbleCube` from a raw array of `u4x2`s.
	/// `a` is the even element, `b` is the odd element.
	/// For example:
	/// (x:0,y:0,z:0) is index 0, element `a`.
	/// (x:15,y:0,z:15) is index 127, element `b`.
	pub fn from_raw(raw: Box<[u8; 2048]>) -> Self {
		NibbleCube(raw)
	}

	/// Sets a value, without clearing what was there previously.
	/// This uses the `u4x2::replace_or` function internally, and shares the same semantics.
	/// This can be used as an optimization to avoid clearing an already cleared value when operating
	/// on a fresh buffer.
	pub fn set_uncleared(&mut self, at: CubePosition, value: u4) {
		let value = value.raw() & 15;

		let (index, shift) = nibble_index(at.yzx() as usize);

		self.0[index] |= value << shift;
	}

	/// Returns a reference to the raw array of `u4x2`s.
	/// `a` is the even element, `b` is the odd element.
	/// For example:
	/// (x:0,y:0,z:0) is index 0, element `a`.
	/// (x:15,y:0,z:15) is index 127, element `b`.
	pub fn raw(&self) -> &[u8; 2048] {
		&self.0
	}

	/// Returns the raw array of `u4x2`s.
	/// `a` is the even element, `b` is the odd element.
	/// For example:
	/// (x:0,y:0,z:0) is index 0, element `a`.
	/// (x:15,y:0,z:15) is index 127, element `b`.
	pub fn into_raw(self) -> Box<[u8; 2048]> {
		self.0
	}
}

impl CubeStorage<u4> for NibbleCube {
	fn get(&self, at: CubePosition) -> u4 {
		let (index, shift) = nibble_index(at.yzx() as usize);

		let single = self.0[index] & (0xF << shift);

		u4::new(single >> shift)
	}

	fn set(&mut self, at: CubePosition, value: u4) {
		let value = value.raw() & 15;

		let (index, shift) = nibble_index(at.yzx() as usize);

		let cleared = !((!self.0[index]) | (0xF << shift));

		self.0[index] = cleared | (value << shift);
	}

	fn fill(&mut self, value: u4) {
		let value = value.raw() & 15;
		let fill = (value << 4) | value;

		for term in self.0.iter_mut() {
			*term = fill;
		}
	}
}

impl Default for NibbleCube {
	fn default() -> Self {
		NibbleCube(Box::new([0; 2048]))
	}
}

impl Debug for NibbleCube {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{:?}", &self.0[..])
	}
}

impl Clone for NibbleCube {
	fn clone(&self) -> Self {
		let mut other = Box::new([0; 2048]);

		other.copy_from_slice(&self.0[..]);

		NibbleCube(other)
	}
}