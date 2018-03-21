use level::anvil::NibbleVec;
use types::position::ChunkPosition;
use std::hash::Hash;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::mem;
use std::fmt::{self, Debug};
use storage::mask::{Mask, ChunkMask};
use storage::packed::{PackedBlockStorage, PackedIndex};

pub trait Target: Eq + Hash + Clone + Debug {}
impl<T> Target for T where T: Eq + Hash + Clone + Debug {}

#[derive(Debug, Clone)]
pub struct Chunk<B, R=NullRecorder> where B: Target, R: Record<ChunkPosition> {
	storage:  PackedBlockStorage<ChunkPosition>,
	palette:  Palette<B>,
	recorder: R,
	bits:     u8
}

impl<B, R> Chunk<B, R> where B: Target, R: Record<ChunkPosition> + Default {
	pub fn new(bits_per_entry: usize, default: B) -> Self {
		Chunk {
			storage: PackedBlockStorage::new(bits_per_entry),
			palette: Palette::new(bits_per_entry, default),
			recorder: R::default(),
			bits: bits_per_entry as u8
		}
	}
}

impl<B, R> Chunk<B, R> where B: Target, R: Record<ChunkPosition> {
	pub fn with_recorder(bits_per_entry: usize, default: B, recorder: R) -> Self {
		Chunk {
			storage: PackedBlockStorage::new(bits_per_entry),
			palette: Palette::new(bits_per_entry, default),
			recorder,
			bits: bits_per_entry as u8
		}
	}

	/// Increases the capacity of this chunk's storage by the specified amount of bits, and returns the old storage for reuse purposes.
	pub fn reserve_bits(&mut self, bits: usize) -> PackedBlockStorage<ChunkPosition> {
		self.palette.reserve_bits(bits);

		let mut replacement_storage = PackedBlockStorage::new(self.storage.bits_per_entry() + bits);

		replacement_storage.clone_from(&self.storage, &self.palette);
		
		mem::swap(&mut self.storage, &mut replacement_storage);

		self.bits += bits as u8;

		replacement_storage
	}
	
	/// Makes sure that a future lookup for the target will succeed, unless the entry has changed since this call.
	pub fn ensure_available(&mut self, target: B) {
		 if let Err(target) = self.palette.try_insert(target) {
		 	self.reserve_bits(1);
		 	self.palette.try_insert(target).expect("There should be room for a new entry, we just made some!");
		 }
	}
	
	pub fn get(&self, position: ChunkPosition) -> PaletteAssociation<B> {
		self.storage.get(position, &self.palette)
	}
	
	// TODO: Methods to work with the palette: pruning, etc.
	
	pub fn palette_mut(&mut self) -> &mut Palette<B> {
		&mut self.palette
	}
	
	pub fn palette(&self) -> &Palette<B> {
		&self.palette
	}
	
	pub fn freeze_read_only(&self) -> (&PackedBlockStorage<ChunkPosition>, &Palette<B>, &R) {
		(&self.storage, &self.palette, &self.recorder)
	}

	pub fn freeze_palette(&mut self) -> (&mut PackedBlockStorage<ChunkPosition>, &Palette<B>, &mut R) {
		(&mut self.storage, &self.palette, &mut self.recorder)
	}
	
	/// Preforms the ensure_available, reverse_lookup, and set calls all in one.
	/// Prefer freezing the palette for larger scale block sets.
	pub fn set_immediate(&mut self, position: ChunkPosition, target: &B) {
		self.ensure_available(target.clone());
		let association = self.palette.reverse_lookup(&target).unwrap();
		
		self.storage.set(position, &association, &mut self.recorder);
	}

	pub fn recorder(&self) -> &R {
		&self.recorder
	}

	pub fn recorder_mut(&mut self) -> &mut R {
		&mut self.recorder
	}
}

impl<R> Chunk<u16, R> where R: Record<ChunkPosition> {
	pub fn anvil_empty(&self) -> bool {
		if let Some(assoc) = self.palette.reverse_lookup(&0) {
			self.storage.get_count(&assoc) == 4096
		} else {
			false
		}
	}

	pub fn to_protocol_section(&self) -> Result<(u8, Vec<i32>, &[u64]), u8> {
		if self.bits > 8 {
			return Err(self.bits); // Only support 8 bits or less, because the palette may be scrambled at higher levels.
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

		Ok((self.bits, palette, &self.storage.raw_storage()))
	}

	/// Returns the Blocks, Metadata, and Add arrays for this chunk.
	/// Returns Err if unable to resolve an association.
	pub fn to_anvil(&self) -> Result<(Vec<i8>, NibbleVec, Option<NibbleVec>), usize> {
		let mut blocks = vec![0; 4096];
		let mut meta = NibbleVec::filled();
		
		let mut need_add = false;
		for entry in self.palette.entries.iter().filter_map(|&f| f) {
			// Can't express Anvil IDs over 4095 without Add. TODO: Utilize Counts.
			if entry > 4095 {
				need_add = true;
			}
		}
		
		if need_add {
			let mut add = NibbleVec::filled();
			
			for index in 0..4096 {
				let position = ChunkPosition::from_yzx(index);
				let association = self.storage.get(position, &self.palette);
				let anvil = association.target().map(|&v| v)?;
				
				    blocks[index as usize] = (anvil >> 4)  as i8;
				meta.set_uncleared(position, (anvil & 0xF) as u8);
				 add.set_uncleared(position, (anvil >> 12) as u8);
			}
			
			Ok((blocks, meta, Some(add)))
		} else {
			for index in 0..4096 {
				let position = ChunkPosition::from_yzx(index);
				let association = self.storage.get(position, &self.palette);
				let anvil = association.target().map(|&v| v)?;
				
				    blocks[index as usize] = (anvil >> 4)  as i8;
				meta.set_uncleared(position, (anvil & 0xF) as u8);
			}
			
			Ok((blocks, meta, None))
		}
	}
}

// TODO: THESE FIELDS SHOULD NOT BE PUBLIC!
#[derive(Debug, Copy, Clone)]
pub struct PaletteAssociation<'p, B> where B: 'p + Target {
	pub palette: &'p Palette<B>,
	pub value: usize
}

impl<'p, B> PaletteAssociation<'p, B> where B: 'p + Target {
	pub fn target(&self) -> Result<&B, usize> {
		self.palette.entries[self.value].as_ref().ok_or(self.value)
	}
	
	pub fn raw_value(&self) -> usize {
		self.value
	}

	pub fn palette(&self) -> &Palette<B> {
		self.palette
	}
}

#[derive(Debug, Clone)]
pub struct Palette<B> where B: Target {
	entries: Vec<Option<B>>,
	reverse: HashMap<B, usize>
}

impl<B> Palette<B> where B: Target {
	pub fn new(bits_per_entry: usize, default: B) -> Self {
		let mut reverse = HashMap::new();
		reverse.insert(default.clone(), 0);

		let mut entries = vec![None; 1<<bits_per_entry];
		entries[0] = Some(default);

		Palette { entries, reverse }
	}
	
	pub fn reserve_bits(&mut self, bits: usize) {
		let additional = (self.entries.len() << bits) - self.entries.len();

		self.entries.reserve(additional);

		for _ in 0..additional {
			self.entries.push(None);
		}
	}
	
	pub fn try_insert(&mut self, target: B) -> Result<usize, B> {
		match self.reverse.entry(target.clone()) {
			Entry::Occupied(occupied) => Ok(*occupied.get()),
			Entry::Vacant(vacant) => {
				let mut idx = None;
				for (index, slot) in self.entries.iter_mut().enumerate() {
					if slot.is_none() {
						*slot = Some(target);
						idx = Some(index);
						break;
					}
				}
				
				match idx {
					Some(index) => {
						vacant.insert(index);
						Ok(index)
					},
					None => Err(vacant.into_key())
				}
			}
		}
	}
	
	/// Replaces the entry at `index` with the target, even if `index` was previously vacant. 
	pub fn replace(&mut self, index: usize, target: B) {
		let old = mem::replace(&mut self.entries[index], Some(target.clone()));
		
		if let Some(old_target) = old {
			let mut other_reference = None;
		
			for (index, entry) in self.entries.iter().enumerate() {
				if let &Some(ref other) = entry {
					if *other == old_target {
						other_reference = Some(index);
						break;
					}
				}
			}
			
			if let Entry::Occupied(mut occ) = self.reverse.entry(old_target) {
				if let Some(other) = other_reference {
					if *occ.get() == index {
						occ.insert(other);
					}
				} else {
					occ.remove();
				}
			}
		}
		
		// Only replace entries in the reverse lookup if they don't exist, otherwise keep the previous entry.
		self.reverse.entry(target).or_insert(index);
	}
	
	/// Gets an association that will reference back to the target. Note that several indices may point to the same target, this returns one of them.
	pub fn reverse_lookup(&self, target: &B) -> Option<PaletteAssociation<B>> {
		self.reverse.get(target).map(|&value| PaletteAssociation { palette: self, value })
	}
	
	pub fn entries(&self) -> &[Option<B>] {
		&self.entries
	}
}

pub trait Record<P> where P: PackedIndex {
	fn record<B>(&mut self, position: P, association: &PaletteAssociation<B>) where B: Target;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct NullRecorder;
impl<P> Record<P> for NullRecorder where P: PackedIndex {
	fn record<B>(&mut self, _: P, _: &PaletteAssociation<B>) where B: Target {}
}

#[derive(Clone, Default)]
pub struct DirtyRecorder {
	mask: ChunkMask,
	any:  bool
}

impl DirtyRecorder {
	pub fn any(&self) -> bool {
		self.any
	}

	pub fn reset_any(&mut self) {
		self.any = false;
	}

	pub fn mask(&self) -> &ChunkMask {
		&self.mask
	}

	pub fn mask_mut(&mut self) -> &mut ChunkMask {
		&mut self.mask
	}
}

impl Record<ChunkPosition> for DirtyRecorder {
	fn record<B>(&mut self, position: ChunkPosition, _: &PaletteAssociation<B>) where B: Target {
		self.any |= !self.mask[position];

		self.mask.set_true(position)
	}
}

impl Debug for DirtyRecorder {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "DirtyRecorder")
	}
}