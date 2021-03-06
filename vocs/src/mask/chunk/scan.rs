use crate::mask::{BitCube, Scan, ScanClear, u1x64};
use crate::position::CubePosition;
use std::cmp;

impl<'a> IntoIterator for Scan<'a, BitCube, CubePosition> {
	type Item = CubePosition;
	type IntoIter = ChunkScan<'a>;

	fn into_iter(self) -> Self::IntoIter {
		ChunkScan::new(self.0)
	}
}

pub struct ChunkScan<'a> {
	mask: &'a BitCube,
	keep: u1x64,
	skip: u8,
	done: bool
}

impl<'a> ChunkScan<'a> {
	pub fn new(mask: &'a BitCube) -> Self {
		let mut scan = ChunkScan {
			mask,
			keep: u1x64::splat(true),
			skip: 0,
			done: false
		};

		scan.fast_forward();

		scan
	}

	fn fast_forward(&mut self) {
		if !(self.mask.blocks()[self.skip as usize] & self.keep).empty() {
			return;
		}

		for (index, block) in (&self.mask.blocks()[self.skip as usize + 1..]).iter().enumerate() {
			if !block.empty() {
				self.keep = u1x64::splat(true);
				self.skip += 1 + index as u8;

				return;
			}
		}

		self.done = true;
	}
}

impl<'a> Iterator for ChunkScan<'a> {
	type Item = CubePosition;

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}

		let block = self.mask.blocks()[self.skip as usize] & self.keep;
		let index = block.first_bit() as u16;

		self.keep = u1x64::from_bits(u64::max_value() << cmp::min(index + 1, 63));
		if index == 63 {
			self.keep = u1x64::splat(false);
		}

		let index = ((self.skip as u16) * 64) | index;

		self.fast_forward();

		Some(CubePosition::from_yzx(index))
	}
}

impl<'a> IntoIterator for ScanClear<'a, BitCube, CubePosition> {
	type Item = CubePosition;
	type IntoIter = ChunkScanClear<'a>;

	fn into_iter(self) -> Self::IntoIter {
		ChunkScanClear::new(self.0)
	}
}

pub struct ChunkScanClear<'a>(&'a mut BitCube);

impl<'a> ChunkScanClear<'a> {
	pub fn new(mask: &'a mut BitCube) -> Self {
		ChunkScanClear(mask)
	}
}

impl<'a> Iterator for ChunkScanClear<'a> {
	type Item = CubePosition;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.pop_first()
	}
}

#[cfg(test)]
mod tests {
	use crate::position::CubePosition;
	use crate::mask::{Mask, BitCube};
	use std::collections::BTreeSet;

	#[test]
	fn test_chunk_scan() {
		for scram in 0..128u32 {
			let mut mask = BitCube::default();
			let mut positions = BTreeSet::new();

			for index in 0..64u32 {
				let pos = scram * 529 + index*507;
				let position = CubePosition::from_yzx((pos as u16) & 4095);

				positions.insert(position);
				mask.set_true(position);
			}

			let expected_vec = positions.iter().map(|x| *x).collect::<Vec<CubePosition>>();
			let created_vec = mask.scan().into_iter().collect::<Vec<CubePosition>>();

			assert_eq!(expected_vec, created_vec);
		}
	}

	#[test]
	fn test_chunk_scan_clear() {
		for scram in 0..128u32 {
			let mut mask = BitCube::default();
			let mut positions = BTreeSet::new();

			for index in 0..64u32 {
				let pos = scram * 529 + index*507;
				let position = CubePosition::from_yzx((pos as u16) & 4095);

				positions.insert(position);
				mask.set_true(position);
			}

			let expected_vec = positions.iter().map(|x| *x).collect::<Vec<CubePosition>>();
			let created_vec = mask.scan_clear().into_iter().collect::<Vec<CubePosition>>();

			assert_eq!(expected_vec, created_vec);

			if mask != BitCube::default() {
				panic!("BitCube::scan_clear did not clear the mask!");
			}
		}
	}
}