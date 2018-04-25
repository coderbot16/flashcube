mod palette;

use std::hash::Hash;
use std::mem;
use std::fmt::Debug;
use packed::{PackedStorage, PackedIndex, Setter};
use position::{ChunkPosition, LayerPosition};

pub use self::palette::Palette;

pub type ChunkIndexed<B> = IndexedStorage<B, ChunkPosition>;
pub type LayerIndexed<B> = IndexedStorage<B, LayerPosition>;

pub trait Target: Eq + Hash + Clone + Debug {}
impl<T> Target for T where T: Eq + Hash + Clone + Debug {}

#[derive(Debug, Clone)]
pub struct IndexedStorage<B, P> where B: Target, P: PackedIndex {
	storage: PackedStorage<P>,
	palette: Palette<B>
}

impl<B, P> IndexedStorage<B, P> where B: Target, P: PackedIndex {
	pub fn new(bits: u8, default: B) -> Self {
		IndexedStorage {
			storage: PackedStorage::new(bits),
			palette: Palette::new(bits, default)
		}
	}

	/// Increases the capacity of this chunk's storage by the specified amount of bits, and returns the old storage for reuse purposes.
	pub fn reserve_bits(&mut self, bits: u8) -> PackedStorage<P> {
		self.palette.expand(bits);

		let mut replacement_storage = PackedStorage::new(self.storage.bits() + bits);

		replacement_storage.clone_from(&self.storage, None, 0);

		mem::replace(&mut self.storage, replacement_storage)
	}
	
	/// Makes sure that a future lookup for the target will succeed, unless the entry has changed since this call.
	pub fn ensure_available(&mut self, target: B) {
		 if let Err(target) = self.palette.try_insert(target) {
		 	self.reserve_bits(1);
		 	self.palette.try_insert(target).expect("There should be room for a new entry, we just made some!");
		 }
	}
	
	pub fn get(&self, position: P) -> &B {
		self.palette.entries()[self.storage.get(position) as usize].as_ref().expect("IndexedStorage is corrupted; A user of freeze_palette has likely violated the API contract!")
	}

	pub fn fill(&mut self, block: B) {
		self.palette.clear();

		if self.palette.entries().len() == 0 {
			self.palette.expand(1);
		}

		self.palette.replace(0, block);
		self.storage.fill(0);
	}

	pub fn palette(&self) -> &Palette<B> {
		&self.palette
	}
	
	pub fn freeze(&self) -> (&PackedStorage<P>, &[Option<B>]) {
		(&self.storage, self.palette.entries())
	}

	/// Freezes the palette, and returns a mutable storage.
	/// Setting invalid values in the PackedStorage will lead to errors.
	/// This is the only API that can set invalid values in the storage.
	/// If only setting one value, then use IndexedStorage::setter instead.
	// TODO: Fix the corruption hole.
	pub fn freeze_palette(&mut self) -> (&mut PackedStorage<P>, &Palette<B>) {
		(&mut self.storage, &self.palette)
	}

	/// Configures a setter to set a certain block in this storage.
	/// This has the same performance cost as set_immediate for a single set,
	/// but is cheaper for multiple sets.
	pub fn setter(&mut self, target: B) -> (Setter<P>, &[Option<B>]) {
		let value = match self.palette.try_insert(target) {
			Err(target) => {
				self.reserve_bits(1);
				self.palette.try_insert(target).expect("There should be room for a new entry, we just made some!")
			},
			Ok(value) => value
		};

		(self.storage.setter(value), self.palette.entries())
	}
	
	/// Preforms the ensure_available, reverse_lookup, and set calls all in one.
	/// Prefer freezing the palette for larger scale block sets, or using a setter.
	pub fn set_immediate(&mut self, position: P, target: &B) {
		self.ensure_available(target.clone());
		let association = self.palette.reverse_lookup(&target).unwrap();
		
		self.storage.set(position, association);
	}

	pub fn bits(&self) -> u8 {
		self.storage.bits()
	}
}

impl ChunkIndexed<u16> {
	pub fn anvil_empty(&self) -> bool {
		/*if let Some(assoc) = self.palette.reverse_lookup(&0) {
			self.storage.get_count(&assoc) == 4096
		} else {
			false
		}*/
		false /*TODO*/
	}

	pub fn to_protocol_section(&self) -> Result<(u8, Vec<i32>, &[u64]), u8> {
		let bits = self.bits();

		if bits > 8 {
			return Err(bits); // Only support 8 bits or less, because the palette may be scrambled at higher levels.
		}

		let mut palette = Vec::with_capacity(self.palette.entries().len());

		let mut skipped = 0;

		for entry in self.palette.entries() {
			match entry {
				&Some(entry) => {
					for _ in 0..skipped {
						palette.push(0);
					}

					skipped = 0;

					palette.push(entry as i32);
				},
				&None => skipped += 1
			}
		}

		Ok((bits, palette, &self.storage.raw_storage()))
	}
}