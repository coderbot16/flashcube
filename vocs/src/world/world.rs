use types::position::{ChunkPosition, LayerPosition};
use world::chunk::{Chunk, Target, Record, PaletteAssociation};
use std::collections::hash_map::{HashMap, Entry, Iter, IterMut};
use std::slice;
use std::ops::{Index, IndexMut};

pub struct Sector<T> {
	chunks: Box<[Option<T>]>,
	present: usize
}

impl<T> Sector<T> where T: Clone {
	pub fn new() -> Self {
		Sector {
			chunks: vec![None; 4096].into_boxed_slice(),
			present: 0
		}
	}
}

impl<T> Sector<T> {
	pub fn set(&mut self, position: ChunkPosition, chunk: T) {
		let target = &mut self.chunks[position.yzx() as usize];

		if target.is_none() {
			self.present += 1;
		}

		*target = Some(chunk);
	}
	
	pub fn remove(&mut self, position: ChunkPosition) -> Option<T> {
		let value = self.chunks[position.yzx() as usize].take();
		
		if value.is_some() {
			self.present -= 1;
		}
		
		value
	}

	/// Gets a mutable reference to an individual element of the sector,
	/// This is not implemented as IndexMut because it may cause the internal present counter to get out of sync.
	pub fn get_mut(&mut self, position: ChunkPosition) -> Option<&mut T> {
		self.chunks[position.yzx() as usize].as_mut()
	}

	pub fn iter(&self) -> slice::Iter<Option<T>> {
		self.chunks.iter()
	}

	// TODO: This can result in the present counter getting out of sync.
	pub fn iter_mut(&mut self) -> slice::IterMut<Option<T>> {
		self.chunks.iter_mut()
	}
	
	pub fn is_empty(&self) -> bool {
		self.present == 0
	}

	pub fn columns(&self) -> SectorColumns<T> {
		SectorColumns {
			region: &self,
			column: LayerPosition::from_zx(0),
			done: false
		}
	}

	pub fn columns_mut(&mut self) -> SectorColumnsMut<T> {
		let slice = &mut self.chunks[..];

		let (s0 , slice) = slice.split_at_mut(256);
		let (s1 , slice) = slice.split_at_mut(256);
		let (s2 , slice) = slice.split_at_mut(256);
		let (s3 , slice) = slice.split_at_mut(256);
		let (s4 , slice) = slice.split_at_mut(256);
		let (s5 , slice) = slice.split_at_mut(256);
		let (s6 , slice) = slice.split_at_mut(256);
		let (s7 , slice) = slice.split_at_mut(256);
		let (s8 , slice) = slice.split_at_mut(256);
		let (s9 , slice) = slice.split_at_mut(256);
		let (s10, slice) = slice.split_at_mut(256);
		let (s11, slice) = slice.split_at_mut(256);
		let (s12, slice) = slice.split_at_mut(256);
		let (s13, slice) = slice.split_at_mut(256);
		let (s14, s15  ) = slice.split_at_mut(256);

		SectorColumnsMut {
			layers: (
				s0 .iter_mut(), s1 .iter_mut(), s2 .iter_mut(), s3 .iter_mut(),
				s4 .iter_mut(), s5 .iter_mut(), s6 .iter_mut(), s7 .iter_mut(),
				s8 .iter_mut(), s9 .iter_mut(), s10.iter_mut(), s11.iter_mut(),
				s12.iter_mut(), s13.iter_mut(), s14.iter_mut(), s15.iter_mut()
			),
			index: 0,
			done: false
		}
	}
}

impl<B, R> Sector<Chunk<B, R>> where B: Target, R: Record<ChunkPosition> {
	pub fn set_block_immediate(&mut self, x: u8, y: u8, z: u8, target: &B) -> Option<()> {
		let (chunk, block) = (
			ChunkPosition::new(x / 16, y / 16, z / 16),
			ChunkPosition::new(x % 16, y % 16, z % 16)
		);

		self.get_mut(chunk).map(|chunk| chunk.set_immediate(block, &target))
	}

	pub fn get_block(&self, x: u8, y: u8, z: u8) -> Option<PaletteAssociation<B>> {
		let (chunk, block) = (
			ChunkPosition::new(x / 16, y / 16, z / 16),
			ChunkPosition::new(x % 16, y % 16, z % 16)
		);

		// TODO: Better error handling.
		self[chunk].as_ref().map(|chunk| chunk.get(block))
	}
}

impl<T> Index<ChunkPosition> for Sector<T> {
	type Output = Option<T>;

	fn index(&self, position: ChunkPosition) -> &Self::Output {
		&self.chunks[position.yzx() as usize]
	}
}

pub struct SectorColumns<'a, T> where T: 'a {
	region: &'a Sector<T>,
	column: LayerPosition,
	done:   bool
}

impl<'a, T> Iterator for SectorColumns<'a, T> where T: 'a {
	type Item = [Option<&'a T>; 16];

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}

		let mut chunks = [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None];

		for y in 0..16 {
			let position = ChunkPosition::from_layer(y, self.column);

			chunks[y as usize] = self.region[position].as_ref();
		}

		if self.column == LayerPosition::from_zx(255) {
			self.done = true;
		} else {
			self.column = LayerPosition::from_zx(self.column.zx() + 1);
		}

		Some(chunks)
	}
}

pub struct SectorColumnsMut<'a, T> where T: 'a {
	layers: (slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>,
			 slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>,
			 slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>,
			 slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>, slice::IterMut<'a, Option<T>>),
	index: u8,
	done:  bool
}

impl<'a, T> Iterator for SectorColumnsMut<'a, T> where T: 'a  {
	type Item = [Option<&'a mut T>; 16];

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}
		
		let chunks = [
			self.layers. 0.next().unwrap().as_mut(),
			self.layers. 1.next().unwrap().as_mut(),
			self.layers. 2.next().unwrap().as_mut(),
			self.layers. 3.next().unwrap().as_mut(),
			self.layers. 4.next().unwrap().as_mut(),
			self.layers. 5.next().unwrap().as_mut(),
			self.layers. 6.next().unwrap().as_mut(),
			self.layers. 7.next().unwrap().as_mut(),
			self.layers. 8.next().unwrap().as_mut(),
			self.layers. 9.next().unwrap().as_mut(),
			self.layers.10.next().unwrap().as_mut(),
			self.layers.11.next().unwrap().as_mut(),
			self.layers.12.next().unwrap().as_mut(),
			self.layers.13.next().unwrap().as_mut(),
			self.layers.14.next().unwrap().as_mut(),
			self.layers.15.next().unwrap().as_mut()
		];

		if self.index == 255 {
			self.done = true;
		} else {
			self.index += 1;
		}

		Some(chunks)
	}
}

pub struct LayerSector<T> {
	chunks: Box<[Option<T>]>,
	present: usize
}

impl<T> LayerSector<T> where T: Clone {
	pub fn new() -> Self {
		LayerSector {
			chunks: vec![None; 256].into_boxed_slice(),
			present: 0
		}
	}
}

impl<T> LayerSector<T> {
	pub fn set(&mut self, position: LayerPosition, chunk: T) {
		let target = &mut self.chunks[position.zx() as usize];

		if target.is_none() {
			self.present += 1;
		}

		*target = Some(chunk);
	}

	pub fn remove(&mut self, position: LayerPosition) -> Option<T> {
		let value = self.chunks[position.zx() as usize].take();

		if value.is_some() {
			self.present -= 1;
		}

		value
	}

	pub fn get(&self, position: LayerPosition) -> Option<&T> {
		self.chunks[position.zx() as usize].as_ref()
	}

	pub fn get_mut(&mut self, position: LayerPosition) -> Option<&mut T> {
		self.chunks[position.zx() as usize].as_mut()
	}

	pub fn is_empty(&self) -> bool {
		self.present == 0
	}
}

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