use std::marker::PhantomData;

// TODO: Separate storage and palette, but still have a PaletteAssociation based API.
use world::chunk::{PaletteAssociation, Target, Palette, Record, NullRecorder};

pub trait PackedIndex: Copy {
	fn entries() -> usize;
	fn from_index(index: usize) -> Self;
	fn to_index(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct PackedBlockStorage<P> where P: PackedIndex {
	storage: Vec<u64>,
	counts: Vec<usize>,
	bits_per_entry: usize,
	bitmask: u64,
	phantom: PhantomData<P>
}

enum Indices {
	Single(usize),
	Double(usize, usize)
}

impl<P> PackedBlockStorage<P> where P: PackedIndex {
	pub fn new(bits_per_entry: usize) -> Self {
		let mut counts = vec![0; 1 << bits_per_entry];
		counts[0] = P::entries();

		PackedBlockStorage {
			storage: vec![0; bits_per_entry * (P::entries() / 64)],
			counts,
			bits_per_entry,
			bitmask: (1 << (bits_per_entry as u64)) - 1,
			phantom: PhantomData
		}
	}

	fn indices(&self, index: usize) -> (Indices, u8) {
		let bit_index = index*self.bits_per_entry;
		// Calculate the indices to the u64 array.
		let start = bit_index / 64;
		let end = ((bit_index + self.bits_per_entry) - 1) / 64;
		let sub_index = (bit_index % 64) as u8;

		// Does the packed sample start and end in the same u64?
		if start==end {
			(Indices::Single(start), sub_index)
		} else {
			(Indices::Double(start, end), sub_index)
		}
	}

	pub fn get_count<B>(&self, association: &PaletteAssociation<B>) -> usize where B: Target {
		self.counts[association.raw_value()]
	}

	pub fn counts(&self) -> &[usize] {
		&self.counts
	}

	pub fn raw_storage(&self) -> &[u64] {
		&self.storage
	}

	pub fn get_raw(&self, position: P) -> usize {
		if self.bits_per_entry == 0 {
			return 0;
		}

		let index = position.to_index();

		let (indices, sub_index) = self.indices(index);

		let raw = match indices {
			Indices::Single(index) => self.storage[index] >> sub_index,
			Indices::Double(start, end) => {
				let end_sub_index = 64 - sub_index;
				(self.storage[start] >> sub_index) | (self.storage[end] << end_sub_index)
			}
		} & self.bitmask;

		raw as usize
	}

	pub fn get<'p, B>(&self, position: P, palette: &'p Palette<B>) -> PaletteAssociation<'p, B> where B: 'p + Target {
		PaletteAssociation {
			palette,
			value: self.get_raw(position)
		}
	}

	pub fn set<B, R>(&mut self, position: P, association: &PaletteAssociation<B>, recorder: &mut R) where B: Target, R: Record<P> {
		if self.bits_per_entry == 0 {
			return;
		}

		let value = association.raw_value() as u64;
		let index = position.to_index();

		let previous = self.get(position, association.palette());
		self.counts[previous.raw_value()] -= 1;
		self.counts[association.raw_value()] += 1;

		if previous.raw_value() != association.raw_value() {
			recorder.record(position, association);
		}

		let (indices, sub_index) = self.indices(index);
		match indices {
			Indices::Single(index) => self.storage[index] = self.storage[index] & !(self.bitmask << sub_index) | (value & self.bitmask) << sub_index,
			Indices::Double(start, end) => {
				let end_sub_index = 64 - sub_index;
				self.storage[start] = self.storage[start] & !(self.bitmask << sub_index)  | (value & self.bitmask) << sub_index;
				self.storage[end]   = self.storage[end] >> end_sub_index << end_sub_index | (value & self.bitmask) >> end_sub_index;
			}
		}
	}

	/// Clones a smaller storage into this larger storage. Both stores must have the same palette.
	pub fn clone_from<B>(&mut self, from: &PackedBlockStorage<P>, palette: &Palette<B>) -> bool where B: Target {
		if from.bits_per_entry < self.bits_per_entry {
			return false;
		}

		let added_bits = from.bits_per_entry - self.bits_per_entry;

		self.counts.clear();

		for count in &from.counts {
			self.counts.push(*count);
		}

		for _ in 0..added_bits {
			let add = self.counts.len();
			self.counts.reserve(add);

			for _ in 0..add {
				self.counts.push(0);
			}
		}

		if added_bits == 0 {
			self.storage.clone_from(&from.storage);
		} else {
			// TODO: Optimize this loop!

			for index in 0..P::entries() {
				let position = P::from_index(index);
				self.set(position, &from.get(position, palette), &mut NullRecorder);
			}
		}

		true
	}

	pub fn bits_per_entry(&self) -> usize {
		self.bits_per_entry
	}
}