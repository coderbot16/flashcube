use mask::{Mask, ChunkMask, Scan, ScanClear};
use position::ChunkPosition;
use std::cmp;

impl<'a> IntoIterator for Scan<'a, ChunkMask, ChunkPosition> {
	type Item = ChunkPosition;
	type IntoIter = ChunkScan<'a>;

	fn into_iter(self) -> Self::IntoIter {
		ChunkScan::new(self.0)
	}
}

pub struct ChunkScan<'a> {
	mask: &'a ChunkMask,
	keep: u64,
	skip: u8,
	done: bool
}

impl<'a> ChunkScan<'a> {
	pub fn new(mask: &'a ChunkMask) -> Self {
		let mut scan = ChunkScan {
			mask,
			keep: u64::max_value(),
			skip: 0,
			done: false
		};

		scan.fast_forward();

		scan
	}

	fn fast_forward(&mut self) {
		if self.mask.blocks()[self.skip as usize] & self.keep != 0 {
			return;
		}

		for (index, block) in (&self.mask.blocks()[self.skip as usize + 1..]).iter().enumerate() {
			if *block != 0 {
				self.keep = u64::max_value();
				self.skip += 1 + index as u8;

				return;
			}
		}

		self.done = true;
	}
}

impl<'a> Iterator for ChunkScan<'a> {
	type Item = ChunkPosition;

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}

		let block = self.mask.blocks()[self.skip as usize] & self.keep;
		let index = block.trailing_zeros() as u16;

		self.keep = u64::max_value() << cmp::min(index + 1, 63);
		if index == 63 {
			self.keep = 0;
		}

		let index = ((self.skip as u16) * 64) | index;

		self.fast_forward();

		Some(ChunkPosition::from_yzx(index))
	}
}

impl<'a> IntoIterator for ScanClear<'a, ChunkMask, ChunkPosition> {
	type Item = ChunkPosition;
	type IntoIter = ChunkScanClear<'a>;

	fn into_iter(self) -> Self::IntoIter {
		ChunkScanClear::new(self.0)
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

#[cfg(test)]
mod tests {
	use position::ChunkPosition;
	use mask::{Mask, ChunkMask};
	use std::collections::BTreeSet;

	#[test]
	fn test_chunk_scan() {
		for scram in 0..128u32 {
			let mut mask = ChunkMask::default();
			let mut positions = BTreeSet::new();

			for index in 0..64u32 {
				let pos = scram * 529 + index*507;
				let position = ChunkPosition::from_yzx((pos as u16) & 4095);

				positions.insert(position);
				mask.set_true(position);
			}

			let mut expected_vec = positions.iter().map(|x| *x).collect::<Vec<ChunkPosition>>();
			let mut created_vec = mask.scan().into_iter().collect::<Vec<ChunkPosition>>();
		}
	}
}