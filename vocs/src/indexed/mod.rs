pub mod palette;

use position::ChunkPosition;
use std::hash::Hash;
use std::mem;
use std::fmt::Debug;
use storage::packed::ChunkPacked;
use storage::indexed::palette::Palette;

pub trait Target: Eq + Hash + Clone + Debug {}
impl<T> Target for T where T: Eq + Hash + Clone + Debug {}

#[derive(Debug, Clone)]
pub struct ChunkIndexed<B> where B: Target {
	storage: ChunkPacked,
	palette: Palette<B>
}

impl<B> ChunkIndexed<B> where B: Target {
	pub fn new(bits: u8, default: B) -> Self {
		ChunkIndexed {
			storage: ChunkPacked::new(bits),
			palette: Palette::new(bits, default)
		}
	}

	/// Increases the capacity of this chunk's storage by the specified amount of bits, and returns the old storage for reuse purposes.
	pub fn reserve_bits(&mut self, bits: u8) -> ChunkPacked {
		self.palette.expand(bits);

		let mut replacement_storage = ChunkPacked::new(self.storage.bits() + bits);

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
	
	pub fn get(&self, position: ChunkPosition) -> Option<&B> {
		self.palette.entries()[self.storage.get(position) as usize].as_ref()
	}
	
	// TODO: Methods to work with the palette: pruning, etc.
	
	pub fn palette_mut(&mut self) -> &mut Palette<B> {
		&mut self.palette
	}
	
	pub fn palette(&self) -> &Palette<B> {
		&self.palette
	}
	
	pub fn freeze_read_only(&self) -> (&ChunkPacked, &Palette<B>) {
		(&self.storage, &self.palette)
	}

	pub fn freeze_palette(&mut self) -> (&mut ChunkPacked, &Palette<B>) {
		(&mut self.storage, &self.palette)
	}
	
	/// Preforms the ensure_available, reverse_lookup, and set calls all in one.
	/// Prefer freezing the palette for larger scale block sets.
	pub fn set_immediate(&mut self, position: ChunkPosition, target: &B) {
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
		unimplemented!()
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