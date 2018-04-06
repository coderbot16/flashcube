use position::{ChunkPosition, LayerPosition};
use storage::mask::{Mask, ChunkMask, LayerMask};
use bit_vec::BitVec;
use std::marker::PhantomData;

pub struct ScanClear<'a, T, P>(pub &'a mut T, pub PhantomData<P>) where T: 'a + Mask<P> + ?Sized;

impl<'a> IntoIterator for ScanClear<'a, ChunkMask, ChunkPosition> {
	type Item = ChunkPosition;
	type IntoIter = ChunkScanClear<'a>;

	fn into_iter(self) -> Self::IntoIter {
		ChunkScanClear::new(self.0)
	}
}

impl<'a> IntoIterator for ScanClear<'a, LayerMask, LayerPosition> {
	type Item = LayerPosition;
	type IntoIter = LayerScanClear<'a>;

	fn into_iter(self) -> Self::IntoIter {
		LayerScanClear::new(self.0)
	}
}

impl<'a> IntoIterator for ScanClear<'a, BitVec, usize> {
	type Item = usize;
	type IntoIter = BitsScanClear<'a>;

	fn into_iter(self) -> Self::IntoIter {
		BitsScanClear::new(self.0)
	}
}

pub struct ChunkScanClear<'a> {
	mask: &'a mut ChunkMask,
	skip: u8,
	done: bool
}

impl<'a> ChunkScanClear<'a> {
	pub fn new(mask: &'a mut ChunkMask) -> Self {
		let mut scan_clear = ChunkScanClear {
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

impl<'a> Iterator for ChunkScanClear<'a> {
	type Item = ChunkPosition;

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}

		let block = self.mask.blocks()[self.skip as usize];
		let index = ((self.skip as u16) * 64) | (block.trailing_zeros() as u16);

		let position = ChunkPosition::from_yzx(index);

		self.mask.set_false(position);
		self.fast_forward();

		Some(position)
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