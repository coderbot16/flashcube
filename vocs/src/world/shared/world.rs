use crate::position::{GlobalChunkPosition, GlobalColumnPosition, GlobalSectorPosition};
use crate::world::shared::{Packed, Guard, SharedSector};
use std::collections::hash_map::{HashMap, Entry, Iter, IterMut};

// TODO: Concurrent Hash Map
pub struct SharedWorld<T> where T: Packed {
	sectors: HashMap<GlobalSectorPosition, SharedSector<T>>
}

impl<T> SharedWorld<T> where T: Packed {
	pub fn new() -> Self {
		SharedWorld {
			sectors: HashMap::new()
		}
	}

	pub fn set(&mut self, position: GlobalChunkPosition, chunk: T) {
		let sector = position.global_sector();
		let inner = position.local_chunk();

		self.sectors.entry(sector).or_insert_with(SharedSector::new).set(inner, chunk);
	}

	pub fn set_column(&mut self, position: GlobalColumnPosition, column: [T; 16]) {
		let sector = position.global_sector();
		let inner = position.local_layer();

		self.sectors.entry(sector).or_insert_with(SharedSector::new).set_column(inner, column);
	}

	pub fn remove(&mut self, position: GlobalChunkPosition) -> Option<T> {
		let sector = position.global_sector();
		let inner = position.local_chunk();

		if let Entry::Occupied(mut occupied) = self.sectors.entry(sector) {
			let value = occupied.get_mut().remove(inner);

			// TODO
			/*if occupied.get().is_empty() {
				occupied.remove();
			}*/

			value
		} else {
			None
		}
	}

	// TODO:
	pub fn get(&self, position: GlobalChunkPosition) -> Option<Guard<T>> {
		let sector = position.global_sector();
		let inner = position.local_chunk();

		self.sectors.get(&sector).and_then(|sector| sector.get(inner))
	}

	/*pub fn get_mut(&mut self, position: GlobalChunkPosition) -> Option<&mut T> {
		let sector = position.global_sector();
		let inner = position.local_chunk();

		self.sectors.get_mut(&sector).and_then(|sector| sector.get_mut(inner))
	}*/

	pub fn get_sector(&self, position: GlobalSectorPosition) -> Option<&SharedSector<T>> {
		self.sectors.get(&position)
	}

	// TODO: Leak
	pub fn get_sector_mut(&mut self, position: GlobalSectorPosition) -> Option<&mut SharedSector<T>> {
		self.sectors.get_mut(&position)
	}

	pub fn get_or_create_sector_mut(&mut self, position: GlobalSectorPosition) -> &mut SharedSector<T> {
		self.sectors.entry(position).or_insert_with(SharedSector::new)
	}

	/*pub fn get_column_mut(&mut self, position: GlobalColumnPosition) -> Option<[&mut T; 16]> {
		let sector = position.global_sector();
		let inner = position.local_layer();

		self.sectors.get_mut(&sector).and_then(|sector| sector.get_column_mut(inner))
	}*/

	pub fn sectors(&self) -> Iter<GlobalSectorPosition, SharedSector<T>> {
		self.sectors.iter()
	}

	pub fn sectors_mut(&mut self) -> IterMut<GlobalSectorPosition, SharedSector<T>> {
		self.sectors.iter_mut()
	}

	pub fn into_sectors(self) -> HashMap<GlobalSectorPosition, SharedSector<T>> {
		self.sectors
	}
}

impl<T> SharedWorld<T> where T: Packed + Default {
	pub fn get_or_create_mut(&mut self, position: GlobalChunkPosition) -> Guard<T> {
		let sector = position.global_sector();
		let inner = position.local_chunk();

		self.sectors.entry(sector).or_insert_with(SharedSector::new).get_or_create(inner)
	}
}