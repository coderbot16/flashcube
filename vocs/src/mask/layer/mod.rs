use mask::Mask;
use component::LayerStorage;
use position::LayerPosition;
use std::ops::Index;

mod scan;

pub use self::scan::*;

// Hackish constants for implementing Index on bit packed structures.
const FALSE_REF: &bool = &false;
const TRUE_REF:  &bool = &true;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct LayerMask([u64; 4]);
impl LayerMask {
	pub fn blocks(&self) -> &[u64; 4] {
		&self.0
	}

	pub fn blocks_mut(&mut self) -> &mut [u64; 4] {
		&mut self.0
	}
}

impl LayerStorage<bool> for LayerMask {
	fn get(&self, position: LayerPosition) -> bool {
		self[position]
	}

	fn is_filled(&self, value: bool) -> bool {
		let term = if value { u64::max_value() } else { 0 };

		self.0 == [term, term, term, term]
	}

	fn set(&mut self, position: LayerPosition, value: bool) {
		let index = position.zx() as usize;

		let array_index = index / 64;
		let shift = index % 64;

		let cleared = self.0[array_index] & !(1 << shift);
		self.0[array_index] = cleared | ((value as u64) << shift)
	}

	fn fill(&mut self, value: bool) {
		if value {
			self.0[0] = u64::max_value();
			self.0[1] = u64::max_value();
			self.0[2] = u64::max_value();
			self.0[3] = u64::max_value();
		} else {
			self.0[0] = 0;
			self.0[1] = 0;
			self.0[2] = 0;
			self.0[3] = 0;
		}
	}
}

impl Mask<LayerPosition> for LayerMask {
	fn set_true(&mut self, position: LayerPosition) {
		let index = position.zx() as usize;

		self.0[index / 64] |= 1 << (index % 64);
	}

	fn set_false(&mut self, position: LayerPosition) {
		let index = position.zx() as usize;

		self.0[index / 64] &= !(1 << (index % 64));
	}

	fn set_or(&mut self, position: LayerPosition, value: bool) {
		let index = position.zx() as usize;

		self.0[index / 64] |= (value as u64) << (index % 64);
	}

	fn count_ones(&self) -> u32 {
		self.0[0].count_ones() + self.0[1].count_ones() + self.0[2].count_ones() + self.0[3].count_ones()
	}

	fn count_zeros(&self) -> u32 {
		self.0[0].count_zeros() + self.0[1].count_zeros() + self.0[2].count_zeros() + self.0[3].count_zeros()
	}
}

impl Index<LayerPosition> for LayerMask {
	type Output = bool;

	fn index(&self, position: LayerPosition) -> &bool {
		let index = position.zx() as usize;

		if (self.0[index / 64] >> (index % 64))&1 == 1 { TRUE_REF } else { FALSE_REF }
	}
}