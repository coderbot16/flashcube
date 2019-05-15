use crate::position::{GlobalSectorPosition, GlobalChunkPosition};
use crate::mask::sparse::recycle::{Recycler, AllocCache};
use crate::mask::{Mask, ChunkMask, Scan, ScanClear};
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
	sectors: HashMap<GlobalSectorPosition, ChunkMask>,
	cache: AllocCache<ChunkMask>
}

impl ChunksMask {
	pub fn new(cache_max_size: usize) -> Self {
		ChunksMask {
			sectors: HashMap::new(),
			cache: AllocCache::new(cache_max_size)
		}
	}

	pub fn sectors(&self) -> Keys<GlobalSectorPosition, ChunkMask> {
		self.sectors.keys()
	}

	pub fn sector(&self, coordinates: GlobalSectorPosition) -> Option<&ChunkMask> {
		self.sectors.get(&coordinates)
	}

	fn require_sector(&mut self, sector: GlobalSectorPosition) -> &mut ChunkMask {
		let cache = &mut self.cache;
		self.sectors.entry(sector).or_insert_with(|| cache.create())
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
	/*fn clear(&mut self) {
		for (_, (mut value, _)) in self.sectors.drain().take(self.cache.remaining_capacity()) {
			value.clear();
			self.cache.destroy(value);
		}

		self.sectors.clear();
	}*/

	fn set_true(&mut self, chunk: GlobalChunkPosition) {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		self.require_sector(sector).set_true(inner);
	}

	fn set_false(&mut self, chunk: GlobalChunkPosition) {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		match self.sectors.entry(sector) {
			Entry::Occupied(mut entry) => {
				{
					let mask = entry.get_mut();

					mask.set_false(inner);

					if !mask.empty() {
						return;
					}
				}

				self.cache.destroy(entry.remove_entry().1)
			},
			Entry::Vacant(_) => return
		}
	}

	fn set_or(&mut self, chunk: GlobalChunkPosition, value: bool) {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		self.require_sector(sector).set_or(inner, value);

		// We don't need to check to see if the mask is empty here.
		// ChunkMask::set_or can either (1) not change the mask, or (2) add another bit.
		// Since the mask can't lose a bit, we don't need to check.
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
		self.sectors.values().fold(0, |state, value| state + value.count_ones() as u32)
	}

	fn count_zeros(&self) -> u32 {
		self.sectors.values().fold(0, |state, value| state + value.count_zeros() as u32)
	}
}

impl Index<GlobalChunkPosition> for ChunksMask {
	type Output = bool;

	fn index(&self, chunk: GlobalChunkPosition) -> &bool {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		self.sectors.get(&sector).map(|mask| &mask[inner]).unwrap_or(FALSE_REF)
	}
}