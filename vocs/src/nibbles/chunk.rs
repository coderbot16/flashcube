use std::fmt::{Debug, Formatter, Result};
use position::ChunkPosition;
use super::{u4, nibble_index};
use component::ChunkStorage;

pub struct ChunkNibbles(Box<[u8; 2048]>);
impl ChunkNibbles {
	/// Sets the lighting value using bitwise OR without clearing the slot first.
	/// This avoids an unnecessary clear when operating on a known-cleared buffer,
	/// but users should prefer `set` instead.
	pub fn set_uncleared(&mut self, at: ChunkPosition, value: u8) {
		let value = value & 15;

		let (index, shift) = nibble_index(at.yzx() as usize);

		self.0[index] |= value << shift;
	}

	pub fn raw(&self) -> &[u8; 2048] {
		&self.0
	}

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