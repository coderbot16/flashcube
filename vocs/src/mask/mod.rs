/// Scans bit collections for set bits, and clears them.
mod scan_clear;

/// Scans bit collections for set bits.
mod scan;

/// Implements sparse masks for blocks, chunks, or layers. This builds on the base masks
/// provided in the base mask package.
pub mod sparse;

/// Contains SpillChunkMask, allowing relative sets on a ChunkMask that may "spill" over into adjacent ChunkMasks.
pub mod spill;

use self::scan::Scan;
use self::scan_clear::ScanClear;
use component::{Component, ChunkStorage, LayerStorage};
pub use self::scan_clear::{ChunkScanClear, LayerScanClear};

use bit_vec::BitVec;
use position::{ChunkPosition, LayerPosition, Offset, Up, Down, PlusX, MinusX, PlusZ, MinusZ};
use std::ops::Index;
use std::u64;

// TODO: SparseIncoming mask: Like SparseMask, but ChunkMask is replaced with IncomingChunkMask.

// Hackish constants for implementing Index on bit packed structures.
const FALSE_REF: &bool = &false;
const TRUE_REF:  &bool = &true;

impl Component for bool {
	type Chunk = ChunkMask;
	type Layer = LayerMask;
	type Bulk = ();
}

pub trait Mask<P>: Index<P, Output=bool> {
	fn set_true(&mut self, index: P);
	fn set_false(&mut self, index: P);

	fn set_or(&mut self, index: P, value: bool);
	fn set(&mut self, index: P, value: bool) {
		if value {
			self.set_true(index)
		} else {
			self.set_false(index)
		}
	}

	fn scan(&self) -> Scan<Self, P> {
		Scan(self, ::std::marker::PhantomData)
	}

	fn scan_clear(&mut self) -> ScanClear<Self, P> {
		ScanClear(self, ::std::marker::PhantomData)
	}

	fn count_ones(&self) -> u32;
	fn count_zeros(&self) -> u32;
}

pub struct ChunkMask(Box<[u64; 64]>);
impl ChunkMask {
	pub fn combine(&mut self, other: &ChunkMask) {
		for (target, other) in self.blocks_mut().iter_mut().zip(other.blocks().iter()) {
			*target = *target | *other;
		}
	}

	pub fn set_neighbors(&mut self, position: ChunkPosition) {
		position.offset(MinusX).map(|at| self.set_true(at));
		position.offset(PlusX ).map(|at| self.set_true(at));
		position.offset(MinusZ).map(|at| self.set_true(at));
		position.offset(PlusZ ).map(|at| self.set_true(at));
		position.offset(Down  ).map(|at| self.set_true(at));
		position.offset(Up    ).map(|at| self.set_true(at));
	}

	pub fn blocks(&self) -> &[u64; 64] {
		&self.0
	}

	pub fn blocks_mut(&mut self) -> &mut [u64; 64] {
		&mut self.0
	}
}

impl ChunkStorage<bool> for ChunkMask {
	fn get(&self, position: ChunkPosition) -> bool {
		self[position]
	}

	fn set(&mut self, position: ChunkPosition, value: bool) {
		<Self as Mask<ChunkPosition>>::set(self, position, value);
	}

	fn fill(&mut self, value: bool) {
		if value {
			for value in self.0.iter_mut() {
				*value = u64::max_value();
			}
		} else {
			for value in self.0.iter_mut() {
				*value = 0;
			}
		}
	}
}

impl Mask<ChunkPosition> for ChunkMask {
	fn set_true(&mut self, position: ChunkPosition) {
		let index = position.yzx() as usize;

		self.0[index / 64] |= 1 << (index % 64);
	}

	fn set_false(&mut self, position: ChunkPosition) {
		let index = position.yzx() as usize;

		self.0[index / 64] &= !(1 << (index % 64));
	}

	fn set_or(&mut self, position: ChunkPosition, value: bool) {
		let index = position.yzx() as usize;

		self.0[index / 64] |= (value as u64) << (index % 64);
	}

	fn set(&mut self, position: ChunkPosition, value: bool) {
		let index = position.yzx() as usize;

		let array_index = index / 64;
		let shift = index % 64;

		let cleared = self.0[array_index] & !(1 << shift);
		self.0[array_index] = cleared | ((value as u64) << shift)
	}

	fn count_ones(&self) -> u32 {
		self.0.iter().fold(0, |state, value| state + value.count_ones())
	}

	fn count_zeros(&self) -> u32 {
		self.0.iter().fold(0, |state, value| state + value.count_zeros())
	}
}

impl Index<ChunkPosition> for ChunkMask {
	type Output = bool;

	fn index(&self, position: ChunkPosition) -> &bool {
		let index = position.yzx() as usize;

		if (self.0[index / 64] >> (index % 64))&1 == 1 { TRUE_REF } else { FALSE_REF }
	}
}

impl Clone for ChunkMask {
	fn clone(&self) -> Self {
		ChunkMask(Box::new([
			self.0[ 0], self.0[ 1], self.0[ 2], self.0[ 3], self.0[ 4], self.0[ 5], self.0[ 6], self.0[ 7], self.0[ 8], self.0[ 9],
			self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15], self.0[16], self.0[17], self.0[18], self.0[19],
			self.0[20], self.0[21], self.0[22], self.0[23], self.0[24], self.0[25], self.0[26], self.0[27], self.0[28], self.0[29],
			self.0[30], self.0[31], self.0[32], self.0[33], self.0[34], self.0[35], self.0[36], self.0[37], self.0[38], self.0[39],
			self.0[40], self.0[41], self.0[42], self.0[43], self.0[44], self.0[45], self.0[46], self.0[47], self.0[48], self.0[49],
			self.0[50], self.0[51], self.0[52], self.0[53], self.0[54], self.0[55], self.0[56], self.0[57], self.0[58], self.0[59],
			self.0[60], self.0[61], self.0[62], self.0[63]
		]))
	}
}

impl Default for ChunkMask {
	fn default() -> Self {
		ChunkMask(Box::new([0; 64]))
	}
}

#[derive(Debug, Default, Clone)]
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
		<Self as Mask<LayerPosition>>::set(self, position, value);
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

	fn set(&mut self, position: LayerPosition, value: bool) {
		let index = position.zx() as usize;

		let array_index = index / 64;
		let shift = index % 64;

		let cleared = self.0[array_index] & !(1 << shift);
		self.0[array_index] = cleared | ((value as u64) << shift)
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