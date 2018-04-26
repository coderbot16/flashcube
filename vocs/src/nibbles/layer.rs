use std::fmt::{Debug, Formatter, Result};
use position::LayerPosition;
use super::{u4, u4x2, nibble_index};
use component::LayerStorage;

pub struct LayerNibbles([u4x2; 128]);
impl LayerNibbles {
	pub fn set_uncleared(&mut self, at: LayerPosition, value: u4) {
		let (index, shift) = nibble_index(at.zx() as usize);

		self.0[index] = self.0[index].replace_or(shift, value);
	}

	pub fn raw(&self) -> &[u4x2; 128] {
		&self.0
	}
}

impl LayerStorage<u4> for LayerNibbles {
	fn get(&self, at: LayerPosition) -> u4 {
		let (index, shift) = nibble_index(at.zx() as usize);

		self.0[index].extract(shift)
	}

	fn is_empty(&self) -> bool {
		for &term in self.0.iter() {
			if term.ba() != 0 {
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