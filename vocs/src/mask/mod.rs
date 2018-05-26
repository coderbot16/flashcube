/// Implements sparse masks for blocks, chunks, or layers. This builds on the base masks
/// provided in the base mask package.
pub mod sparse;

pub mod layer;
pub mod chunk;

pub use self::layer::*;
pub use self::chunk::*;

use component::{Component, ChunkStorage, LayerStorage};

use bit_vec::BitVec;
use position::{ChunkPosition, LayerPosition, Offset, dir};
use std::ops::Index;
use std::u64;
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

pub struct Scan     <'a, T, P>(pub &'a     T, pub PhantomData<P>) where T: 'a + Mask<P> + ?Sized;
pub struct ScanClear<'a, T, P>(pub &'a mut T, pub PhantomData<P>) where T: 'a + Mask<P> + ?Sized;