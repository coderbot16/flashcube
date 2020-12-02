use std::collections::HashMap;
use crate::packed::setter::Setter;

struct Indices {
	start: usize,
	end: usize
}

pub trait PackedIndex: Copy {
	fn size_factor() -> usize;
	fn from_usize(usize: usize) -> Self;
	fn to_usize(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct PackedStorage<P>(Box<[u64]>, ::std::marker::PhantomData<P>) where P: PackedIndex;

impl<P> PackedStorage<P> where P: PackedIndex {
	pub fn new(bits: u8) -> Self {
		PackedStorage (
			vec![0; (bits as usize) * P::size_factor()].into_boxed_slice(),
			::std::marker::PhantomData
		)
	}

	fn indices(&self, index: usize, bits: u8) -> (Indices, u8) {
		let bits = bits as usize;

		let bit_index = index * bits;
		// Calculate the indices to the u64 array.
		let start = bit_index / 64;
		let end = ((bit_index + bits) - 1) / 64;
		let sub_index = (bit_index % 64) as u8;

		(Indices { start, end }, sub_index)
	}

	/// Calculates the bit count from the internal array size.
	pub fn bits(&self) -> u8 {
		(self.0.len() / P::size_factor()) as u8
	}

	pub fn raw_storage(&self) -> &[u64] {
		&self.0
	}

	pub fn get(&self, position: P) -> u32 {
		if self.0.len() == 0 {
			return 0;
		}

		let bits = self.bits();
		let bitmask = (1u64 << bits) - 1;
		let index = position.to_usize();

		let (indices, sub_index) = self.indices(index, bits);

		let mut raw = self.0[indices.start] >> sub_index;

		if indices.start != indices.end {
			raw |= self.0[indices.end] << (64 - sub_index);
		}

		(raw & bitmask) as u32
	}

	pub fn set(&mut self, position: P, value: u32) {
		if self.0.len() == 0 {
			return;
		}

		let bits = self.bits();
		let bitmask = (1u64 << bits) - 1;
		let value = value as u64 & bitmask;
		let index = position.to_usize();

		let (indices, sub_index) = self.indices(index, bits);

		self.0[indices.start] = self.0[indices.start] & !(bitmask << sub_index) | value << sub_index;

		if indices.start != indices.end {
			let end_sub_index = 64 - sub_index;
			self.0[indices.end] = self.0[indices.end] >> end_sub_index << end_sub_index | value >> end_sub_index;
		}
	}

	pub fn setter(&mut self, value: u32) -> Setter<P> {
		Setter::new(self, value)
	}

	pub fn clear(&mut self) {
		for value in self.0.iter_mut() {
			*value = 0;
		}
	}

	pub fn fill(&mut self, value: u32) {
		if value == 0 {
			self.clear();
			return;
		}

		// TODO: Possibly repeat values into a bit pattern?
		for index in 0..P::size_factor()*64 {
			self.set(P::from_usize(index), value)
		}
	}

	/// Clones from another storage into this storage using the provided translation table.
	/// Alternately, truncates the values.
	/// Any missing translations are replaced with the default.
	pub fn clone_from(&mut self, from: &PackedStorage<P>, translation: Option<&HashMap<u32, u32>>, default: u32) {
		if self.0.len() == from.0.len() {
			self.0.clone_from(&from.0);
			return;
		}

		match translation {
			Some(translation) => for index in 0..P::size_factor()*64 {
				let position = P::from_usize(index);
				self.set(position, *translation.get(&from.get(position)).unwrap_or(&default));
			},
			None => for index in 0..P::size_factor()*64 {
				let position = P::from_usize(index);
				self.set(position, from.get(position));
			}
		}
	}
}