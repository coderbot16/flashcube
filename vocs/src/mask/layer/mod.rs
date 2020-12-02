use crate::mask::Mask;
use crate::component::LayerStorage;
use crate::position::LayerPosition;
use std::ops::{BitOrAssign, Index};

mod scan;

pub use self::scan::*;

// Hackish constants for implementing Index on bit packed structures.
const FALSE_REF: &bool = &false;
const TRUE_REF:  &bool = &true;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct BitLayer([u64; 4]);
impl BitLayer {
	pub fn blocks(&self) -> &[u64; 4] {
		&self.0
	}

	pub fn blocks_mut(&mut self) -> &mut [u64; 4] {
		&mut self.0
	}
}

impl LayerStorage<bool> for BitLayer {
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

impl Mask<LayerPosition> for BitLayer {
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

impl Index<LayerPosition> for BitLayer {
	type Output = bool;

	fn index(&self, position: LayerPosition) -> &bool {
		let index = position.zx() as usize;

		if (self.0[index / 64] >> (index % 64))&1 == 1 { TRUE_REF } else { FALSE_REF }
	}
}

impl BitOrAssign<BitLayer> for BitLayer {
	fn bitor_assign(&mut self, other: BitLayer) {
		self.0[0] |= other.0[0];
		self.0[1] |= other.0[1];
		self.0[2] |= other.0[2];
		self.0[3] |= other.0[3];
	}
}

impl BitOrAssign<&BitLayer> for BitLayer {
	fn bitor_assign(&mut self, other: &BitLayer) {
		self.0[0] |= other.0[0];
		self.0[1] |= other.0[1];
		self.0[2] |= other.0[2];
		self.0[3] |= other.0[3];
	}
}

#[cfg(test)]
mod test {
	use crate::position::LayerPosition;
	use crate::component::LayerStorage;
	use crate::mask::{BitLayer, Mask};

	#[test]
	fn test_plain_set() {
		for position in LayerPosition::enumerate() {
			let mut mask = BitLayer::default();

			mask.set(position, true);
			assert!(mask.get(position), "Mask set failed on index: {:?}", position);
		}
	}

	#[test]
	fn test_mixed_set() {
		let mut mask = BitLayer::default();

		for position in LayerPosition::enumerate() {
			mask.set(position, should_set(position));
		}

		for position in LayerPosition::enumerate() {
			assert_eq!(mask.get(position), should_set(position), "Mask set failed on index: {:?}", position);
		}

		display_mask(&mask);
	}

	#[test]
	fn test_fill() {
		{
			let mut mask = BitLayer::default();

			for position in LayerPosition::enumerate() {
				mask.set(position, should_set(position));
			}

			assert!(!mask.is_filled(true), "test_fill: NotFilled failed");
			assert!(!mask.is_filled(false), "test_fill: NotFilled failed");
		}

		{
			let mut mask = BitLayer::default();

			for position in LayerPosition::enumerate() {
				mask.set_true(position);
			}

			assert!(mask.is_filled(true), "test_fill: Filled(True) failed");
			assert!(!mask.is_filled(false), "test_fill: Filled(True) failed");
		}

		{
			let mut mask = BitLayer::default();

			for position in LayerPosition::enumerate() {
				mask.set_false(position);
			}

			assert!(!mask.is_filled(true), "test_fill: Filled(False) failed");
			assert!(mask.is_filled(false), "test_fill: Filled(False) failed");
		}
	}

	fn should_set(position: LayerPosition) -> bool {
		((position.zx() as u64) * 32181) % 13 < 2
	}

	fn display_mask(mask: &BitLayer) {
		for z in 0..16 {
			for x in 0..16 {
				let position = LayerPosition::new(x, z);
				print!("{}", if mask.get(position) {'*'} else {' '});

			}

			println!();
		}
	}
}