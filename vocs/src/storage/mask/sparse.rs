use position::{ChunkPosition, GlobalSectorPosition, GlobalColumnPosition, GlobalChunkPosition};
use storage::mask::pool::Pool;
use storage::mask::{Mask, ChunkMask, LayerMask};
use storage::mask::scan::Scan;
use storage::mask::scan_clear::ScanClear;
use std::collections::HashMap;
use std::collections::hash_map::{Entry, Iter, Keys, IterMut};
use std::ops::Index;

const FALSE_REF: &bool = &false;

pub struct ColumnsMask(HashMap<GlobalSectorPosition, LayerMask>);

impl ColumnsMask {
	pub fn new() -> Self {
		ColumnsMask(HashMap::new())
	}

	pub fn regions(&self) -> Iter<GlobalSectorPosition, LayerMask> {
		self.0.iter()
	}

	// TODO: If the user clears the masks returned by iter_mut, the ColumnsMask will never remove them.
	pub fn sectors_mut(&mut self) -> IterMut<GlobalSectorPosition, LayerMask> {
		self.0.iter_mut()
	}

	pub fn sector(&self, coordinates: GlobalSectorPosition) -> Option<&LayerMask> {
		self.0.get(&coordinates)
	}

	pub fn clear_sector(&mut self, coordinates: GlobalSectorPosition) {
		self.0.remove(&coordinates);
	}

	pub fn fill_sector(&mut self, coordinates: GlobalSectorPosition) {
		let mut mask = LayerMask::default();
		mask.fill();

		self.0.insert(coordinates, mask);
	}
}

impl Mask<GlobalColumnPosition> for ColumnsMask {
	fn clear(&mut self) {
		self.0.clear();
	}

	fn set_false(&mut self, column: GlobalColumnPosition) {
		let (sector, position) = (column.global_sector(), column.local_layer());

		if let Entry::Occupied(mut entry) = self.0.entry(sector) {
			let remove = {
				let mask = entry.get_mut();

				mask.set_false(position);
				mask.count_ones() == 0
			};

			if remove {
				entry.remove();
			}
		}
	}

	fn set_true(&mut self, column: GlobalColumnPosition) {
		let (sector, position) = (column.global_sector(), column.local_layer());

		self.0.entry(sector).or_insert(LayerMask::default()).set_true(position);
	}

	fn set_or(&mut self, column: GlobalColumnPosition, value: bool) {
		if value {
			self.set_true(column);
		}
	}

	fn scan(&self) -> Scan<Self, GlobalColumnPosition> {
		// TODO: Scanning sparse maps has a non deterministic order. Return regions instead?
		unimplemented!("No clear / logical way to scan a sparse map yet...")
	}

	fn scan_clear(&mut self) -> ScanClear<Self, GlobalColumnPosition> {
		// TODO: Scanning sparse maps has a non deterministic order. Return regions instead?
		unimplemented!("No clear / logical way to scan a sparse map yet...")
	}

	fn count_ones(&self) -> u32 {
		self.0.values().fold(0, |state, value| state + value.count_ones())
	}

	fn count_zeros(&self) -> u32 {
		self.0.values().fold(0, |state, value| state + value.count_zeros())
	}
}

impl Index<GlobalColumnPosition> for ColumnsMask {
	type Output = bool;

	fn index(&self, column: GlobalColumnPosition) -> &bool {
		let (sector, inner) = (column.global_sector(), column.local_layer());

		self.0.get(&sector).map(|sector| &sector[inner]).unwrap_or(FALSE_REF)
	}
}

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
	regions: HashMap<GlobalSectorPosition, usize>,
	pool: Pool<ChunkPosition, ChunkMask>
}

impl ChunksMask {
	pub fn new(start_size: usize, max_size: usize) -> Self {
		ChunksMask {
			regions: HashMap::new(),
			pool: Pool::new(start_size, max_size)
		}
	}

	pub fn regions(&self) -> Keys<GlobalSectorPosition, usize> {
		self.regions.keys()
	}

	pub fn sector(&self, coordinates: GlobalSectorPosition) -> Option<&ChunkMask> {
		self.regions.get(&coordinates).map(|&index| &self.pool.pool[index].0)
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
		self.regions.clear();
		self.pool.clear();
	}

	fn set_true(&mut self, chunk: GlobalChunkPosition) {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		let index = match self.regions.entry(sector) {
			Entry::Occupied(occupied) => *occupied.into_mut(),
			Entry::Vacant(vacant) => match self.pool.alloc() {
				Some(index) => { vacant.insert(index); index },
				None => unimplemented!("I don't support allocation failure yet, because I'm so overconfident!") // TODO: Fix / remove allocation failure.
			}
		};

		let mask = &mut self.pool.pool[index];

		mask.1 += (!mask.0[inner]) as u16;
		mask.0.set_true(inner);

		debug_assert_eq!(mask.0.count_ones() as u16, mask.1, "Mask count tracking and true mask count out of sync!");
	}

	fn set_false(&mut self, chunk: GlobalChunkPosition) {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		let (remove, index) = if let Some(&index) = self.regions.get(&sector) {
			let mask = &mut self.pool.pool[index];

			mask.1 -= mask.0[inner] as u16;
			mask.0.set_false(inner);

			debug_assert_eq!(mask.0.count_ones() as u16, mask.1, "Mask count tracking and true mask count out of sync!");

			(mask.1 == 0, index)
		} else {
			return;
		};

		if remove {
			self.regions.remove(&sector);
			self.pool.free(index);
		}
	}

	fn set_or(&mut self, chunk: GlobalChunkPosition, value: bool) {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		let index = match self.regions.entry(sector) {
			Entry::Occupied(occupied) => *occupied.into_mut(),
			Entry::Vacant(vacant) => if value {
				match self.pool.alloc() {
					Some(index) => {
						vacant.insert(index);
						index
					},
					None => unimplemented!("I don't support allocation failure yet, because I'm so overconfident!") // TODO: Fix / remove allocation failure.
				}
			} else {
				return;
			}
		};


		let mask = &mut self.pool.pool[index];

		mask.1 += ((!mask.0[inner]) & value) as u16;
		mask.0.set_or(inner, value);

		debug_assert_eq!(mask.0.count_ones() as u16, mask.1, "Mask count tracking and true mask count out of sync!");
	}

	fn scan(&self) -> Scan<Self, GlobalChunkPosition> {
		// TODO: Scanning sparse maps has a non deterministic order. Return regions instead?
		unimplemented!("No clear / logical way to scan a sparse map yet...")
	}

	fn scan_clear(&mut self) -> ScanClear<Self, GlobalChunkPosition> {
		// TODO: Scanning sparse maps has a non deterministic order. Return regions instead?
		unimplemented!("No clear / logical way to scan a sparse map yet...")
	}

	fn count_ones(&self) -> u32 {
		self.regions.values().fold(0, |state, &value| state + (self.pool.pool[value].1 as u32))
	}

	fn count_zeros(&self) -> u32 {
		self.regions.values().fold(0, |state, &value| state + (self.pool.pool[value].1 as u32))
	}
}

impl Index<GlobalChunkPosition> for ChunksMask {
	type Output = bool;

	fn index(&self, chunk: GlobalChunkPosition) -> &bool {
		let (sector, inner) = (chunk.global_sector(), chunk.local_chunk());

		self.regions.get(&sector).map(|&index| &self.pool.pool[index].0[inner]).unwrap_or(FALSE_REF)
	}
}

/*fn split_coords(coords: GlobalChunkPosition) -> ((i32, i32), ChunkPosition) {
	let sector = (coords.0 >> 4, coords.2 >> 4);
	let inner = ChunkPosition::new((coords.0 & 15) as u8, coords.1, (coords.2 & 15) as u8);

	(sector, inner)
}*/