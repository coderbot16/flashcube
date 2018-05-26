use mask::{Mask, LayerMask, ScanClear};
use position::LayerPosition;

impl<'a> IntoIterator for ScanClear<'a, LayerMask, LayerPosition> {
	type Item = LayerPosition;
	type IntoIter = LayerScanClear<'a>;

	fn into_iter(self) -> Self::IntoIter {
		LayerScanClear::new(self.0)
	}
}

pub struct LayerScanClear<'a> {
	mask: &'a mut LayerMask,
	skip: u8,
	done: bool
}

impl<'a> LayerScanClear<'a> {
	pub fn new(mask: &'a mut LayerMask) -> Self {
		let mut scan_clear = LayerScanClear {
			mask,
			skip: 0,
			done: false
		};

		scan_clear.fast_forward();

		scan_clear
	}

	fn fast_forward(&mut self) {
		for (index, block) in (&self.mask.blocks()[self.skip as usize..]).iter().enumerate() {
			if *block != 0 {
				self.skip += index as u8;
				return;
			}
		}

		self.done = true;
	}
}

impl<'a> Iterator for LayerScanClear<'a> {
	type Item = LayerPosition;

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}

		let block = self.mask.blocks()[self.skip as usize];
		let index = ((self.skip as u8) * 64) | (block.trailing_zeros() as u8);

		let position = LayerPosition::from_zx(index);

		self.mask.set_false(position);
		self.fast_forward();

		Some(position)
	}
}