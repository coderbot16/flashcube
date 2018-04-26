use std::fmt::{Debug, Formatter, Result};
use position::LayerPosition;
use super::{u4, nibble_index};
use component::LayerStorage;

pub struct LayerNibbles([u8; 128]);
impl LayerNibbles {
	pub fn set_uncleared(&mut self, at: LayerPosition, value: u8) {
		let value = value & 15;

		let (index, shift) = nibble_index(at.zx() as usize);

		self.0[index] |= value << shift;
	}

	pub fn raw(&self) -> &[u8; 128] {
		&self.0
	}
}

impl LayerStorage<u4> for LayerNibbles {
	fn set(&mut self, at: LayerPosition, value: u4) {
		let value = value.raw() & 15;

		let (index, shift) = nibble_index(at.zx() as usize);

		let cleared = !((!self.0[index]) | (0xF << shift));

		self.0[index] = cleared | (value << shift);
	}

	fn get(&self, at: LayerPosition) -> u4 {
		let (index, shift) = nibble_index(at.zx() as usize);

		let single = self.0[index] & (0xF << shift);

		u4::new(single >> shift)
	}

	fn fill(&mut self, value: u4) {
		let value = value.raw() & 15;
		let fill = (value << 4) | value;

		for term in &mut self.0 as &mut [u8] {
			*term = fill;
		}
	}
}

impl Default for LayerNibbles {
	fn default() -> Self {
		LayerNibbles([0; 128])
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