use storage::indexed::Target;
use fxhash::FxHashMap;
use std::collections::hash_map::Entry;
use std::mem;

/// A palette that provides a two-way mapping for efficient access.
/// This implementation does no allocation automatically, the only allocation happens in
/// `new`, `expand`, and `try_shrink`.
#[derive(Debug, Clone)]
pub struct Palette<B> where B: Target {
	entries: Box<[Option<B>]>,
	reverse: FxHashMap<B, u32>
}

impl<B> Palette<B> where B: Target {
	pub fn new(bits: u8, default: B) -> Self {
		let mut reverse = FxHashMap::default();
		reverse.insert(default.clone(), 0);

		let mut entries = vec![None; 1<<bits].into_boxed_slice();
		entries[0] = Some(default);

		Palette { entries, reverse }
	}

	pub fn expand(&mut self, extra_bits: u8) -> Box<[Option<B>]> {
		let mut entries = vec![None; self.entries.len()<<extra_bits].into_boxed_slice();

		for (index, entry) in self.entries.iter_mut().enumerate() {
			entries[index] = entry.take();
		}

		mem::replace(&mut self.entries, entries)
	}

	/// Tries to shrink the palette without remapping any elements.
	pub fn try_shrink(&mut self) -> Option<(Box<[Option<B>]>, u8)> {
		let mut half_size = self.entries.len() / 2;
		let mut bits = 0;

		'outer:
		while half_size > 0 {
			for entry in &self.entries[half_size..] {
				if !entry.is_none() {
					break 'outer;
				}
			}

			bits += 1;
			half_size /= 2;
		}

		if bits == 0 {
			None
		} else {


			Some((unimplemented!(), bits))
		}
	}

	pub fn try_insert(&mut self, target: B) -> Result<u32, B> {
		match self.reverse.entry(target.clone()) {
			Entry::Occupied(occupied) => Ok(*occupied.get()),
			Entry::Vacant(vacant) => {
				let mut idx = None;
				for (index, slot) in self.entries.iter_mut().enumerate() {
					if slot.is_none() {
						*slot = Some(target);
						idx = Some(index as u32);
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
	pub fn replace(&mut self, index: u32, target: B) {
		let old = mem::replace(&mut self.entries[index as usize], Some(target.clone()));

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
						occ.insert(other as u32);
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
	pub fn reverse_lookup(&self, target: &B) -> Option<u32> {
		self.reverse.get(target).map(|x| *x)
	}

	pub fn entries(&self) -> &[Option<B>] {
		&self.entries
	}
}