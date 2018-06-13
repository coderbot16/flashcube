use position::{GlobalChunkPosition, GlobalColumnPosition, GlobalSectorPosition, LayerPosition};
use world::sector::Sector;
use std::collections::hash_map::{HashMap, Entry, Iter, IterMut};
use indexed::{Target, ChunkIndexed};
use view::{QuadMut, ColumnMut};
use splitmut::SplitMut;

pub struct World<T> {
	sectors: HashMap<GlobalSectorPosition, Sector<T>>
}

impl<T> World<T> {
	pub fn new() -> Self {
		World {
			sectors: HashMap::new()
		}
	}
	
	pub fn set(&mut self, position: GlobalChunkPosition, chunk: T) {
		let sector = position.global_sector();
		let inner = position.local_chunk();
		
		self.sectors.entry(sector).or_insert(Sector::new()).set(inner, chunk);
	}

	pub fn set_column(&mut self, position: GlobalColumnPosition, column: [T; 16]) {
		let sector = position.global_sector();
		let inner = position.local_layer();

		self.sectors.entry(sector).or_insert(Sector::new()).set_column(inner, column);
	}

	pub fn remove(&mut self, position: GlobalChunkPosition) -> Option<T> {
		let sector = position.global_sector();
		let inner = position.local_chunk();
		
		if let Entry::Occupied(mut occupied) = self.sectors.entry(sector) {
			let value = occupied.get_mut().remove(inner);
			
			if occupied.get().is_empty() {
				occupied.remove();
			}
			
			value
		} else {
			None
		}
	}
	
	pub fn get(&self, position: GlobalChunkPosition) -> Option<&T> {
		let sector = position.global_sector();
		let inner = position.local_chunk();

		self.sectors.get(&sector).and_then(|sector| sector[inner].as_ref())
	}

	pub fn get_mut(&mut self, position: GlobalChunkPosition) -> Option<&mut T> {
		let sector = position.global_sector();
		let inner = position.local_chunk();

		self.sectors.get_mut(&sector).and_then(|sector| sector.get_mut(inner))
	}

	pub fn get_column_mut(&mut self, position: GlobalColumnPosition) -> Option<[&mut T; 16]> {
		let sector = position.global_sector();
		let inner = position.local_layer();

		self.sectors.get_mut(&sector).and_then(|sector| sector.get_column_mut(inner))
	}

	pub fn sectors(&self) -> Iter<GlobalSectorPosition, Sector<T>> {
		self.sectors.iter()
	}

	pub fn sectors_mut(&mut self) -> IterMut<GlobalSectorPosition, Sector<T>> {
		self.sectors.iter_mut()
	}

	pub fn into_sectors(self) -> HashMap<GlobalSectorPosition, Sector<T>> {
		self.sectors
	}
}

impl<B> World<ChunkIndexed<B>> where B: Target {
	pub fn get_quad_mut(&mut self, position: GlobalColumnPosition) -> Option<QuadMut<B>> {
		let sector = position.global_sector();
		let inner = position.local_layer();

		// TODO: Overflow checking for the edge case

		match (inner.x() == 15, inner.z() == 15) {
			(false, false) => return self.sectors.get_mut(&sector).and_then(|sector| sector.get_quad_mut(inner)),
			(true, false) => {
				let (primary, plus_x) = self.sectors.get2_mut(
					&sector,
					&GlobalSectorPosition::new(sector.x() + 1, sector.z()),
				);

				let (primary, plus_x) = match (primary, plus_x) {
					(Ok(primary), Ok(plus_x)) => (primary, plus_x),
					_ => return None
				};

				let (primary, plus_z) = match primary.get2_column_mut(LayerPosition::new(15, inner.z()), LayerPosition::new(15, inner.z() + 1)) {Some(x) => x, None => return None};
				let (plus_x, plus_xz) = match plus_x.get2_column_mut(LayerPosition::new(0, inner.z()), LayerPosition::new(0, inner.z() + 1)) {Some(x) => x, None => return None};

				Some(QuadMut([ColumnMut(primary), ColumnMut(plus_x), ColumnMut(plus_z), ColumnMut(plus_xz)]))
			},
			(false, true) => {
				let (primary, plus_z) = self.sectors.get2_mut(
					&sector,
					&GlobalSectorPosition::new(sector.x(), sector.z() + 1)
				);

				let (primary, plus_z) = match (primary, plus_z) {
					(Ok(primary), Ok(plus_z)) => (primary, plus_z),
					_ => return None
				};

				let (primary, plus_x) = match primary.get2_column_mut(LayerPosition::new(inner.x(), 15), LayerPosition::new(inner.x() + 1, 15)) {Some(x) => x, None => return None};
				let (plus_z, plus_xz) = match plus_z.get2_column_mut(LayerPosition::new(inner.x(), 0), LayerPosition::new(inner.x() + 1, 0)) {Some(x) => x, None => return None};

				Some(QuadMut([ColumnMut(primary), ColumnMut(plus_x), ColumnMut(plus_z), ColumnMut(plus_xz)]))
			},
			(true, true) => {
				let (primary, plus_x, plus_z, plus_xz) = self.sectors.get4_mut(
					&sector,
					&GlobalSectorPosition::new(sector.x() + 1, sector.z()),
					&GlobalSectorPosition::new(sector.x(), sector.z() + 1),
					&GlobalSectorPosition::new(sector.x() + 1, sector.z() + 1)
				);

				let (primary, plus_x, plus_z, plus_xz) = match (primary, plus_x, plus_z, plus_xz) {
					(Ok(primary), Ok(plus_x), Ok(plus_z), Ok(plus_xz)) => (
						primary.get_column_mut(LayerPosition::new(15, 15)).map(ColumnMut),
						plus_x.get_column_mut(LayerPosition::new(0, 15)).map(ColumnMut),
						plus_z.get_column_mut(LayerPosition::new(15, 0)).map(ColumnMut),
						plus_xz.get_column_mut(LayerPosition::new(0, 0)).map(ColumnMut)
					),
					_ => return None
				};

				match (primary, plus_x, plus_z, plus_xz) {
					(Some(primary), Some(plus_x), Some(plus_z), Some(plus_xz)) => Some(QuadMut([primary, plus_x, plus_z, plus_xz])),
					_ => None
				}
			}
		}
	}
}

// TODO: Add test for Columns/ColumnsMut returning 256 results