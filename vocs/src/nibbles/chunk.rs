use std::fmt::{Debug, Formatter, Result};
use crate::position::ChunkPosition;
use super::{u4, nibble_index};
use crate::component::ChunkStorage;

/// A 16x16 collection of nibbles (`u4`s).
/// Indexed with ChunkPosition.
pub struct ChunkNibbles(Box<[u8; 2048]>);
impl ChunkNibbles {
	/// Creates a `ChunkNibbles` from a raw array of `u4x2`s.
	/// `a` is the even element, `b` is the odd element.
	/// For example:
	/// (x:0,y:0,z:0) is index 0, element `a`.
	/// (x:15,y:0,z:15) is index 127, element `b`.
	pub fn from_raw(raw: Box<[u8; 2048]>) -> Self {
		ChunkNibbles(raw)
	}

	/// Sets a value, without clearing what was there previously.
	/// This uses the `u4x2::replace_or` function internally, and shares the same semantics.
	/// This can be used as an optimization to avoid clearing an already cleared value when operating
	/// on a fresh buffer.
	pub fn set_uncleared(&mut self, at: ChunkPosition, value: u4) {
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

impl ChunkStorage<u4> for ChunkNibbles {
	fn get(&self, at: ChunkPosition) -> u4 {
		let (index, shift) = nibble_index(at.yzx() as usize);

		let single = self.0[index] & (0xF << shift);

		u4::new(single >> shift)
	}

	fn set(&mut self, at: ChunkPosition, value: u4) {
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

impl Default for ChunkNibbles {
	fn default() -> Self {
		ChunkNibbles(Box::new([0; 2048]))
	}
}

impl Debug for ChunkNibbles {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{:?}", &self.0[..])
	}
}

impl Clone for ChunkNibbles {
	fn clone(&self) -> Self {
		let mut other = Box::new([0; 2048]);

		other.copy_from_slice(&self.0[..]);

		ChunkNibbles(other)
	}
}