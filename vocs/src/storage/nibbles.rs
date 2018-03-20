use std::fmt::{Debug, Formatter, Result};
use position::{ChunkPosition, LayerPosition};

/// Returns the chunk_yzx index into a nibble array. Returns in the form (index, shift).
fn nibble_index(index: usize) -> (usize, u8) {
	(index >> 1, ((index & 1) as u8) << 2)
}

pub struct ChunkNibbles([u8; 2048]);
impl ChunkNibbles {
	pub fn new() -> Self {
		ChunkNibbles([0; 2048])
	}

	pub fn boxed() -> Box<Self> {
		Box::new(ChunkNibbles([0; 2048]))
	}

	/// Clears every value value to 0. Equivalent to `fill(0)`
	pub fn clear(&mut self) {
		for term in &mut self.0 as &mut [u8] {
			*term = 0;
		}
	}

	/// Sets every value value to `value`.
	pub fn fill(&mut self, value: u8) {
		let value = value & 15;
		let fill = (value << 4) | value;

		for term in &mut self.0 as &mut [u8] {
			*term = fill;
		}
	}

	pub fn set(&mut self, at: ChunkPosition, value: u8) {
		let value = value & 15;

		let (index, shift) = nibble_index(at.yzx() as usize);

		let cleared = !((!self.0[index]) | (0xF << shift));

		self.0[index] = cleared | (value << shift);
	}

	/// Sets the lighting value using bitwise OR without clearing the slot first.
	/// This avoids an unnecessary clear when operating on a known-cleared buffer,
	/// but users should prefer `set` instead.
	pub fn set_uncleared(&mut self, at: ChunkPosition, value: u8) {
		let value = value & 15;

		let (index, shift) = nibble_index(at.yzx() as usize);

		self.0[index] |= value << shift;
	}

	pub fn get(&self, at: ChunkPosition) -> u8 {
		let (index, shift) = nibble_index(at.yzx() as usize);

		let single = self.0[index] & (0xF << shift);

		single >> shift
	}

	pub fn raw(&self) -> &[u8; 2048] {
		&self.0
	}
}

impl Debug for ChunkNibbles {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{:?}", &self.0[..])
	}
}

impl Clone for ChunkNibbles {
	fn clone(&self) -> Self {
		let mut other = [0; 2048];

		other.copy_from_slice(&self.0[..]);

		ChunkNibbles(other)
	}
}

pub struct LayerNibbles([u8; 128]);
impl LayerNibbles {
	pub fn new() -> Self {
		LayerNibbles([0; 128])
	}

	/// Clears every value value to 0. Equivalent to `fill(0)`
	pub fn clear(&mut self) {
		for term in &mut self.0 as &mut [u8] {
			*term = 0;
		}
	}

	/// Sets every value value to `value`.
	pub fn fill(&mut self, value: u8) {
		let value = value & 15;
		let fill = (value << 4) | value;

		for term in &mut self.0 as &mut [u8] {
			*term = fill;
		}
	}

	pub fn set(&mut self, at: LayerPosition, value: u8) {
		let value = value & 15;

		let (index, shift) = nibble_index(at.zx() as usize);

		let cleared = !((!self.0[index]) | (0xF << shift));

		self.0[index] = cleared | (value << shift);
	}

	pub fn set_uncleared(&mut self, at: LayerPosition, value: u8) {
		let value = value & 15;

		let (index, shift) = nibble_index(at.zx() as usize);

		self.0[index] |= value << shift;
	}

	pub fn get(&self, at: LayerPosition) -> u8 {
		let (index, shift) = nibble_index(at.zx() as usize);

		let single = self.0[index] & (0xF << shift);

		single >> shift
	}

	pub fn raw(&self) -> &[u8; 128] {
		&self.0
	}
}

impl Debug for LayerNibbles {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{:?}", &self.0[..])
	}
}

impl Clone for LayerNibbles {
	fn clone(&self) -> Self {
		let mut other = [0; 128];

		other.copy_from_slice(&self.0[..]);

		LayerNibbles(other)
	}
}