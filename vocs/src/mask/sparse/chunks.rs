use position::{GlobalSectorPosition, GlobalChunkPosition};
use mask::sparse::recycle::{Recycler, AllocCache};
use mask::{Mask, ChunkMask};
use mask::scan::Scan;
use mask::scan_clear::ScanClear;
use std::collections::HashMap;
use std::collections::hash_map::{Entry, Keys};
use std::ops::Index;

const FALSE_REF: &bool = &false;

/// A sparse mask for marking entire chunks (16x16x16 cubes).
/// For individual blocks, use BlocksMask.
/// This only supports chunks up to Y=15, a 16 high chunk stack.
/// This mirrors the current Anvil implementation in Minecraft, but
/// does not support true cubic chunks.
/// In this implementation, a ChunkMask represents the chunks in a Sector.
/// Non present ChunkMasks are all filled with 0s.
/// While it may appear that this is another false world abstraction,
/// it is actually appropriate as a sparse mask.
pub struct ChunksMask {
	sectors: HashMap<GlobalSectorPosition, (ChunkMask, u16)>,
	cache: AllocCache<ChunkMask>
}

impl ChunksMask {
	pub fn new(cache_max_size: usize) -> Self {
		ChunksMask {
			sectors: HashMap::new(),
			cache: AllocCache::new(cache_max_size)
		}
	}

	pub fn sectors(&self) -> Keys<GlobalSectorPosition, (ChunkMask, u16)> {
		self.sectors.keys()
	}

	pub fn sector(&self, coordinates: GlobalSectorPosition) -> Option<&ChunkMask> {
		self.sectors.get(&coordinates).map(|v| &v.0)
	}

	fn require_sector(&mut self, sector: GlobalSectorPosition) -> &mut (ChunkMask, u16) {
		let cache = &mut self.cache;
		self.sectors.entry(sector).or_insert_with(|| (cache.create(), 0))
	}

	/*pub fn set_neighbors(&mut self, coords: GlobalChunkPosition) {
		self.set_true((coords.0 + 1, coords.1,     coords.2    ));
		self.set_true((coords.0 - 1, coords.1,     coords.2    ));
		self.set_true((coords.0,     coords.1,     coords.2 + 1));
		self.set_true((coords.0,     coords.1,     coords.2 - 1));

		if coords.1 < 255 {
			self.set_true((coords.0,     coords.1 + 1, coords.2    ));
		}

		if coords.1 > 0 {
			self.set_true((coords.0,     coords.1 - 1, coords.2    ));
		}
	}*/
}

impl Mask<GlobalChunkPosition> for ChunksMask {
	fn clear(&mut self) {
		for (_, (mut value, _)) in self.sectors.drain().take(self.cache.remaining_capacity()) {
			value.clear();
			self.cache.destroy(value);
		}

		self.sectors.clear();
	}

	fn set_true(&mut self, chunk: GlobalChunkPosition) {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		let (ref mut mask, ref mut count) = *self.require_sector(sector);

		*count += (!mask[inner]) as u16;
		mask.set_true(inner);

		debug_assert_eq!(mask.count_ones() as u16, *count, "Mask count tracking and true mask count out of sync!");
	}

	fn set_false(&mut self, chunk: GlobalChunkPosition) {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		match self.sectors.entry(sector) {
			Entry::Occupied(mut entry) => {
				{
					let (ref mut mask, ref mut count) = *entry.get_mut();

					*count -= mask[inner] as u16;
					mask.set_false(inner);

					debug_assert_eq!(mask.count_ones() as u16, *count, "Mask count tracking and true mask count out of sync!");

					if *count > 0 {
						return;
					}
				}

				self.cache.destroy((entry.remove_entry().1).0)
			},
			Entry::Vacant(_) => return
		}
	}

	fn set_or(&mut self, chunk: GlobalChunkPosition, value: bool) {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		let (ref mut mask, ref mut count) = *self.require_sector(sector);

		*count += ((!mask[inner]) & value) as u16;
		mask.set_or(inner, value);

		debug_assert_eq!(mask.count_ones() as u16, *count, "Mask count tracking and true mask count out of sync!");
	}

	fn scan(&self) -> Scan<Self, GlobalChunkPosition> {
		// TODO: Scanning sparse maps has a non deterministic order. Return sectors instead?
		unimplemented!("No clear / logical way to scan a sparse map yet...")
	}

	fn scan_clear(&mut self) -> ScanClear<Self, GlobalChunkPosition> {
		// TODO: Scanning sparse maps has a non deterministic order. Return sectors instead?
		unimplemented!("No clear / logical way to scan a sparse map yet...")
	}

	fn count_ones(&self) -> u32 {
		self.sectors.values().fold(0, |state, value| state + value.1 as u32)
	}

	fn count_zeros(&self) -> u32 {
		self.sectors.values().fold(0, |state, value| state + (512 - value.1 as u32))
	}
}

impl Index<GlobalChunkPosition> for ChunksMask {
	type Output = bool;

	fn index(&self, chunk: GlobalChunkPosition) -> &bool {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		self.sectors.get(&sector).map(|&(ref mask, _)| &mask[inner]).unwrap_or(FALSE_REF)
	}
}