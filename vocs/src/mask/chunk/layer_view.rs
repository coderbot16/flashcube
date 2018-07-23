use mask::{u1x64, LayerMask};
use position::LayerPosition;
use component::*;

pub struct LayerZxMut<'l> {
	layer: &'l mut [u1x64]
}

impl<'l> LayerZxMut<'l> {
	pub fn from_slice(layer: &'l mut [u1x64]) -> Self {
		assert_eq!(layer.len(), 4);

		LayerZxMut { layer }
	}

	pub fn combine(&mut self, other: &LayerMask) {
		assert_eq!(self.layer.len(), 4);

		self.layer[0] |= u1x64::from_bits(other.blocks()[0]);
		self.layer[1] |= u1x64::from_bits(other.blocks()[1]);
		self.layer[2] |= u1x64::from_bits(other.blocks()[2]);
		self.layer[3] |= u1x64::from_bits(other.blocks()[3]);
	}
}

impl<'l> LayerStorage<bool> for LayerZxMut<'l> {
	fn get(&self, position: LayerPosition) -> bool {
		let index = position.zx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		self.layer[block_index].extract(sub_index as u8)
	}

	fn is_filled(&self, value: bool) -> bool {
		let term = u1x64::splat(value);

		self.layer == &[term, term, term, term]
	}

	fn set(&mut self, position: LayerPosition, value: bool) {
		let index = position.zx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		let block = self.layer[block_index].replace(sub_index as u8, value);

		self.layer[block_index] = block;
	}

	fn fill(&mut self, value: bool) {
		let value = u1x64::splat(value);

		assert_eq!(self.layer.len(), 4);

		self.layer[0] = value;
		self.layer[1] = value;
		self.layer[2] = value;
		self.layer[3] = value;
	}
}