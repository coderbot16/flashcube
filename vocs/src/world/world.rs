use position::ChunkPosition;
use world::sector::Sector;
use std::collections::hash_map::{HashMap, Entry, Iter, IterMut};

pub struct World<T> where T: Clone {
	regions: HashMap<(i32, i32), Sector<T>>
}

impl<T> World<T> where T: Clone {
	pub fn new() -> Self {
		World {
			regions: HashMap::new()
		}
	}
	
	pub fn set(&mut self, coords: (i32, u8, i32), chunk: T) {
		let (region, inner) = Self::split_coords(coords);
		
		self.regions.entry(region).or_insert(Sector::new()).set(inner, chunk);
	}
	
	pub fn remove(&mut self, coords: (i32, u8, i32)) -> Option<T> {
		let (region, inner) = Self::split_coords(coords);
		
		if let Entry::Occupied(mut occupied) = self.regions.entry(region) {
			let value = occupied.get_mut().remove(inner);
			
			if occupied.get().is_empty() {
				occupied.remove();
			}
			
			value
		} else {
			None
		}
	}
	
	pub fn get(&self, coords: (i32, u8, i32)) -> Option<&T> {
		let (region, inner) = Self::split_coords(coords);
		
		self.regions.get(&region).and_then(|region| region[inner].as_ref())
	}
	
	pub fn get_mut(&mut self, coords: (i32, u8, i32)) -> Option<&mut T> {
		let (region, inner) = Self::split_coords(coords);
		
		self.regions.get_mut(&region).and_then(|region| region.get_mut(inner))
	}

	pub fn regions(&self) -> Iter<(i32, i32), Sector<T>> {
		self.regions.iter()
	}

	pub fn regions_mut(&mut self) -> IterMut<(i32, i32), Sector<T>> {
		self.regions.iter_mut()
	}

	pub fn into_regions(self) -> HashMap<(i32, i32), Sector<T>> {
		self.regions
	}
	
	fn split_coords(coords: (i32, u8, i32)) -> ((i32, i32), ChunkPosition) {
		let region = (coords.0 >> 4, coords.2 >> 4);
		let inner = ChunkPosition::new((coords.0 & 15) as u8, coords.1, (coords.2 & 15) as u8);
		
		(region, inner)
	}
}

// TODO: Add test for Columns/ColumnsMut returning 256 results