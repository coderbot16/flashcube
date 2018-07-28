/// Implements sparse masks for blocks, chunks, or layers. This builds on the base masks
/// provided in the base mask package.
pub mod sparse;

pub mod layer;
pub mod chunk;

pub use self::layer::*;
pub use self::chunk::*;

use component::Component;

use std::ops::{Index, BitOrAssign, BitOr, BitAndAssign, BitAnd, Not};
use std::marker::PhantomData;

// TODO: SparseIncoming mask: Like SparseMask, but ChunkMask is replaced with IncomingChunkMask.

impl Component for bool {
	type Chunk = ChunkMask;
	type Layer = LayerMask;
	type Bulk = ();
}

pub trait Mask<P>: Index<P, Output=bool> {
	fn set_true(&mut self, index: P);
	fn set_false(&mut self, index: P);

	fn set_or(&mut self, index: P, value: bool);

	fn scan(&self) -> Scan<Self, P> {
		Scan(self, ::std::marker::PhantomData)
	}

	fn scan_clear(&mut self) -> ScanClear<Self, P> {
		ScanClear(self, ::std::marker::PhantomData)
	}

	fn count_ones(&self) -> u32;
	fn count_zeros(&self) -> u32;
}

pub struct Scan     <'a, T, P>(pub &'a     T, pub PhantomData<P>) where T: 'a + Mask<P> + ?Sized;
pub struct ScanClear<'a, T, P>(pub &'a mut T, pub PhantomData<P>) where T: 'a + Mask<P> + ?Sized;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Default)]
pub struct u1x64(u64);

impl u1x64 {
	#[inline]
	pub fn from_bits(bits: u64) -> Self {
		u1x64(bits)
	}

	#[inline]
	pub fn splat(value: bool) -> Self {
		// TODO: Optimized version?

		u1x64 (
			match value {
				false => 0,
				true => u64::max_value()
			}
		)
	}

	#[inline]
	pub fn extract(self, index: u8) -> bool {
		let index = index & 63;

		(self.0 >> index) & 1 != 0
	}

	#[inline]
	pub fn clear(self, index: u8) -> Self {
		let index = index & 63;

		u1x64(self.0 & !(1 << index))
	}

	#[inline]
	pub fn set(self, index: u8) -> Self {
		let index = index & 63;

		u1x64(self.0 | (1 << index))
	}

	#[inline]
	pub fn replace_or(self, index: u8, value: bool) -> Self {
		let index = index & 63;
		let bit = value as u64;

		u1x64(self.0 | (bit << index))
	}

	#[inline]
	pub fn replace(self, index: u8, value: bool) -> Self {
		let index = index & 63;
		let bit = value as u64;

		let cleared = self.0 & !(1 << index);

		u1x64(cleared | (bit << index))
	}

	#[inline]
	pub fn count_ones(self) -> u32 {
		self.0.count_ones()
	}

	#[inline]
	pub fn count_zeros(self) -> u32 {
		self.0.count_zeros()
	}

	#[inline]
	pub fn empty(self) -> bool {
		self.0 == 0
	}

	#[inline]
	pub fn first_bit(self) -> u8 {
		self.0.trailing_zeros() as u8
	}

	#[inline]
	pub fn to_bits(self) -> u64 {
		self.0
	}
}

impl BitOrAssign for u1x64 {
	#[inline]
	fn bitor_assign(&mut self, rhs: Self) {
		self.0 |= rhs.0;
	}
}

impl BitOr for u1x64 {
	type Output = Self;

	#[inline]
	fn bitor(self, rhs: Self) -> Self::Output {
		u1x64(self.0 | rhs.0)
	}
}

impl BitAndAssign for u1x64 {
	#[inline]
	fn bitand_assign(&mut self, rhs: Self) {
		self.0 &= rhs.0;
	}
}

impl BitAnd for u1x64 {
	type Output = Self;

	#[inline]
	fn bitand(self, rhs: Self) -> Self::Output {
		u1x64(self.0 & rhs.0)
	}
}

impl Not for u1x64 {
	type Output = Self;

	#[inline]
	fn not(self) -> Self::Output {
		u1x64(!self.0)
	}
}