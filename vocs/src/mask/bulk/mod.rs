use bit_vec::BitVec;
use mask::{Mask, Scan, ScanClear};

mod scan;

pub use self::scan::*;

impl Mask<usize> for BitVec<u32> {
	/*fn clear(&mut self) {
		BitVec::clear(self);
	}*/

	fn set_true(&mut self, index: usize) {
		self.set(index, true);
	}

	fn set_false(&mut self, index: usize) {
		self.set(index, false);
	}

	fn set_or(&mut self, index: usize, value: bool) {
		let real_value = self[index] | value;

		self.set(index, real_value);
	}

	fn set(&mut self, index: usize, value: bool) {
		BitVec::set(self, index, value);
	}

	fn scan(&self) -> Scan<Self, usize> {
		Scan(self, ::std::marker::PhantomData)
	}

	fn scan_clear(&mut self) -> ScanClear<Self, usize> {
		ScanClear(self, ::std::marker::PhantomData)
	}

	fn count_ones(&self) -> u32 {
		self.blocks().fold(0, |state, value| state + value.count_ones())
	}

	fn count_zeros(&self) -> u32 {
		self.blocks().fold(0, |state, value| state + value.count_zeros())
	}
}