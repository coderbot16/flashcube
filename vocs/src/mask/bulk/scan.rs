use mask::{Mask, ScanClear};
use bit_vec::BitVec;

impl<'a> IntoIterator for ScanClear<'a, BitVec, usize> {
	type Item = usize;
	type IntoIter = BitsScanClear<'a>;

	fn into_iter(self) -> Self::IntoIter {
		BitsScanClear::new(self.0)
	}
}

pub struct BitsScanClear<'a> {
	mask: &'a mut BitVec<u32>,
	skip: usize,
	done: bool
}

impl<'a> BitsScanClear<'a> {
	pub fn new(mask: &'a mut BitVec) -> Self {
		let mut scan_clear = BitsScanClear {
			mask,
			skip: 0,
			done: false
		};

		scan_clear.fast_forward();

		scan_clear
	}

	fn fast_forward(&mut self) {
		for (index, block) in (&self.mask.storage()[self.skip..]).iter().enumerate() {
			if *block != 0 {
				self.skip += index;
				return;
			}
		}

		self.done = true;
	}
}

impl<'a> Iterator for BitsScanClear<'a> {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}

		let block = self.mask.storage()[self.skip];
		let index = (self.skip * 32) | (block.trailing_zeros() as usize);

		self.mask.set(index, false);
		self.fast_forward();

		Some(index)
	}
}