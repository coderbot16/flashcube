use crate::position::{ChunkPosition, LayerPosition};
use crate::indexed::{ChunkIndexed, Target};
use crate::mask::{Mask, ChunkMask};
use crate::view::{ColumnMut, QuadMut};
use std::slice;
use std::ops::Index;

pub struct Sector<T> {
	chunks: Box<[Option<T>]>,
	present: ChunkMask
}

impl<T> Sector<T> {
	pub fn new() -> Self {
		let mut chunks = Vec::with_capacity(4096);

		for _ in 0..4096 {
			chunks.push(None);
		}

		Sector { chunks: chunks.into_boxed_slice(), present: ChunkMask::default() }
	}

	pub fn set(&mut self, position: ChunkPosition, chunk: T) {
		let target = &mut self.chunks[position.yzx() as usize];

		if target.is_none() {
			self.present.set_true(position);
		}

		*target = Some(chunk);
	}

	pub fn pop_first(&mut self) -> Option<(ChunkPosition, T)> {
		self.present.pop_first().and_then(|position| self.chunks[position.yzx() as usize].take().map(|chunk| (position, chunk)))
	}

	pub fn set_column(&mut self, position: LayerPosition, column: [T; 16]) {
		// TODO: This is hackish, and needs a heap allocation. Find a better way!
		// Or, wait for slice patterns.

		let mut chunks = (Box::new(column) as Box<[_]>).into_vec();

		for (index, chunk) in chunks.drain(..).enumerate() {
			let position = ChunkPosition::from_layer(index as u8, position);

			self.set(position, chunk);
		}
	}

	pub fn remove(&mut self, position: ChunkPosition) -> Option<T> {
		let value = self.chunks[position.yzx() as usize].take();

		if value.is_some() {
			self.present.set_false(position);
		}

		value
	}

	pub fn layers_mut(&mut self) -> (&mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>], &mut [Option<T>]) {
		let slice = &mut self.chunks;

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

		(s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15)
	}

	/// Gets a mutable reference to an individual element of the sector,
	/// This is not implemented as IndexMut because it would cause the internal present mask to get out of sync.
	pub fn get_mut(&mut self, position: ChunkPosition) -> Option<&mut T> {
		self.chunks[position.yzx() as usize].as_mut()
	}

	pub fn enumerate_present(&self) -> SectorEnumeratePresent<T> {
		unimplemented!()
	}

	pub fn enumerate_present_mut(&self) -> SectorEnumeratePresentMut<T> {
		unimplemented!()
	}

	pub fn iter(&self) -> slice::Iter<Option<T>> {
		self.chunks.iter()
	}

	// TODO: This can result in the present mask getting out of sync.
	pub fn iter_mut(&mut self) -> slice::IterMut<Option<T>> {
		self.chunks.iter_mut()
	}

	pub fn is_empty(&self) -> bool {
		self.present.empty()
	}

	pub fn count_sectors(&self) -> u32 {
		self.present.count_ones()
	}

	pub fn columns(&self) -> SectorColumns<T> {
		SectorColumns {
			iterator: self.enumerate_columns()
		}
	}

	pub fn enumerate_columns(&self) -> SectorEnumerateColumns<T> {
		SectorEnumerateColumns {
			sector: &self,
			column: LayerPosition::from_zx(0),
			done: false
		}
	}

	pub fn columns_mut(&mut self) -> SectorColumnsMut<T> {
		let s = self.layers_mut();

		SectorColumnsMut {
			layers: (
				s.0 .iter_mut(), s.1 .iter_mut(), s.2 .iter_mut(), s.3 .iter_mut(),
				s.4 .iter_mut(), s.5 .iter_mut(), s.6 .iter_mut(), s.7 .iter_mut(),
				s.8 .iter_mut(), s.9 .iter_mut(), s.10.iter_mut(), s.11.iter_mut(),
				s.12.iter_mut(), s.13.iter_mut(), s.14.iter_mut(), s.15.iter_mut()
			),
			index: 0,
			done: false
		}
	}

	pub fn get_column_mut(&mut self, position: LayerPosition) -> Option<[&mut T; 16]> {
		let index = position.zx() as usize;
		let s = self.layers_mut();

		let chunks = (
			s.0[index].as_mut(), s.1[index].as_mut(), s.2[index].as_mut(), s.3[index].as_mut(),
			s.4[index].as_mut(), s.5[index].as_mut(), s.6[index].as_mut(), s.7[index].as_mut(),
			s.8[index].as_mut(), s.9[index].as_mut(), s.10[index].as_mut(), s.11[index].as_mut(),
			s.12[index].as_mut(), s.13[index].as_mut(), s.14[index].as_mut(), s.15[index].as_mut()
		);

		match chunks {
			(Some(c0), Some(c1), Some(c2), Some(c3),
				Some(c4), Some(c5), Some(c6), Some(c7),
				Some(c8), Some(c9), Some(c10), Some(c11),
				Some(c12), Some(c13), Some(c14), Some(c15))
			=> Some([c0, c1, c2, c3, c4, c5, c6, c7, c8, c9, c10, c11, c12, c13, c14, c15]),
			_ => None
		}
	}

	pub fn get2_column_mut(&mut self, a: LayerPosition, b: LayerPosition) -> Option<([&mut T; 16], [&mut T; 16])> {
		if a == b {
			return None;
		}

		let a = a.zx() as usize;
		let b = b.zx() as usize;
		let s = self.layers_mut();

		// get2 for a slice with no unsafe code. This uses split_at_mut.
		fn get2_safe<T>(s: &mut [T], a: usize, b: usize) -> (&mut T, &mut T) {
			assert_ne!(a, b);

			if a < b {
				let (low, high) = s.split_at_mut(b);

				(&mut low[a], &mut high[0])
			} else {
				let (low, high) = s.split_at_mut(a);

				(&mut high[0], &mut low[b])
			}
		}

		let (c0a,  c0b ) = match get2_safe(s.0,  a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c1a,  c1b ) = match get2_safe(s.1,  a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c2a,  c2b ) = match get2_safe(s.2,  a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c3a,  c3b ) = match get2_safe(s.3,  a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c4a,  c4b ) = match get2_safe(s.4,  a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c5a,  c5b ) = match get2_safe(s.5,  a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c6a,  c6b ) = match get2_safe(s.6,  a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c7a,  c7b ) = match get2_safe(s.7,  a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c8a,  c8b ) = match get2_safe(s.8,  a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c9a,  c9b ) = match get2_safe(s.9,  a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c10a, c10b) = match get2_safe(s.10, a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c11a, c11b) = match get2_safe(s.11, a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c12a, c12b) = match get2_safe(s.12, a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c13a, c13b) = match get2_safe(s.13, a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c14a, c14b) = match get2_safe(s.14, a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };
		let (c15a, c15b) = match get2_safe(s.15, a, b) { (&mut Some(ref mut a), &mut Some(ref mut b)) => (a, b), _ => return None };

		Some((
			[c0a, c1a, c2a, c3a, c4a, c5a, c6a, c7a, c8a, c9a, c10a, c11a, c12a, c13a, c14a, c15a],
			[c0b, c1b, c2b, c3b, c4b, c5b, c6b, c7b, c8b, c9b, c10b, c11b, c12b, c13b, c14b, c15b]
		))
	}

	fn get4_column_mut(&mut self, a: LayerPosition, b: LayerPosition, c: LayerPosition, d: LayerPosition) -> Option<([&mut T; 16], [&mut T; 16], [&mut T; 16], [&mut T; 16])> {
		let a = a.zx() as usize;
		let b = b.zx() as usize;
		let c = c.zx() as usize;
		let d = d.zx() as usize;

		assert!(a < b && b < c && c < d);

		let s = self.layers_mut();

		// get4 for a slice with no unsafe code. This uses split_at_mut.
		fn get4_safe<T>(s: &mut [T], a: usize, b: usize, c: usize, d: usize) -> (&mut T, &mut T, &mut T, &mut T) {
			let (low, b_start) = s.split_at_mut(b);
			let (b_start, c_start) = b_start.split_at_mut(c - b);
			let (c_start, d_start) = c_start.split_at_mut(d - c);

			(&mut low[a], &mut b_start[0], &mut c_start[0], &mut d_start[0])
		}

		let (c0a,  c0b,  c0c,  c0d ) = match get4_safe(s.0,  a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c1a,  c1b,  c1c,  c1d ) = match get4_safe(s.1,  a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c2a,  c2b,  c2c,  c2d ) = match get4_safe(s.2,  a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c3a,  c3b,  c3c,  c3d ) = match get4_safe(s.3,  a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c4a,  c4b,  c4c,  c4d ) = match get4_safe(s.4,  a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c5a,  c5b,  c5c,  c5d ) = match get4_safe(s.5,  a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c6a,  c6b,  c6c,  c6d ) = match get4_safe(s.6,  a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c7a,  c7b,  c7c,  c7d ) = match get4_safe(s.7,  a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c8a,  c8b,  c8c,  c8d ) = match get4_safe(s.8,  a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c9a,  c9b,  c9c,  c9d ) = match get4_safe(s.9,  a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c10a, c10b, c10c, c10d) = match get4_safe(s.10, a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c11a, c11b, c11c, c11d) = match get4_safe(s.11, a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c12a, c12b, c12c, c12d) = match get4_safe(s.12, a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c13a, c13b, c13c, c13d) = match get4_safe(s.13, a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c14a, c14b, c14c, c14d) = match get4_safe(s.14, a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };
		let (c15a, c15b, c15c, c15d) = match get4_safe(s.15, a, b, c, d) { (&mut Some(ref mut a), &mut Some(ref mut b), &mut Some(ref mut c), &mut Some(ref mut d)) => (a, b, c, d), _ => return None };

		Some((
			[c0a, c1a, c2a, c3a, c4a, c5a, c6a, c7a, c8a, c9a, c10a, c11a, c12a, c13a, c14a, c15a],
			[c0b, c1b, c2b, c3b, c4b, c5b, c6b, c7b, c8b, c9b, c10b, c11b, c12b, c13b, c14b, c15b],
			[c0c, c1c, c2c, c3c, c4c, c5c, c6c, c7c, c8c, c9c, c10c, c11c, c12c, c13c, c14c, c15c],
			[c0d, c1d, c2d, c3d, c4d, c5d, c6d, c7d, c8d, c9d, c10d, c11d, c12d, c13d, c14d, c15d]
		))
	}
}

impl<T> Sector<T> where T: Default {
	pub fn get_or_create_mut(&mut self, position: ChunkPosition) -> &mut T {
		let present = &mut self.present;
		self.chunks[position.yzx() as usize].get_or_insert_with(|| { present.set_true(position); T::default() })
	}
}

impl<B> Sector<ChunkIndexed<B>> where B: Target {
	pub fn set_block_immediate(&mut self, x: u8, y: u8, z: u8, target: &B) -> Option<()> {
		let (chunk, block) = (
			ChunkPosition::new(x / 16, y / 16, z / 16),
			ChunkPosition::new(x % 16, y % 16, z % 16)
		);

		self.get_mut(chunk).map(|chunk| chunk.set_immediate(block, &target))
	}

	pub fn get_block(&self, x: u8, y: u8, z: u8) -> Option<&B> {
		let (chunk, block) = (
			ChunkPosition::new(x / 16, y / 16, z / 16),
			ChunkPosition::new(x % 16, y % 16, z % 16)
		);

		self[chunk].as_ref().map(|chunk| chunk.get(block))
	}

	pub fn get_quad_mut(&mut self, position: LayerPosition) -> Option<QuadMut<B>> {
		self.get4_column_mut(
			position,
			LayerPosition::new(position.x() + 1, position.z()),
			LayerPosition::new(position.x(), position.z() + 1),
			LayerPosition::new(position.x() + 1, position.z() + 1)
		).map(|(primary, plus_x, plus_z, plus_xz)|
			QuadMut([ColumnMut(primary), ColumnMut(plus_x), ColumnMut(plus_z), ColumnMut(plus_xz)])
		)
	}
}

impl<T> Index<ChunkPosition> for Sector<T> {
	type Output = Option<T>;

	fn index(&self, position: ChunkPosition) -> &Self::Output {
		&self.chunks[position.yzx() as usize]
	}
}

pub struct SectorColumns<'a, T> where T: 'a {
	iterator: SectorEnumerateColumns<'a, T>
}

impl<'a, T> Iterator for SectorColumns<'a, T> where T: 'a {
	type Item = [Option<&'a T>; 16];

	fn next(&mut self) -> Option<Self::Item> {
		self.iterator.next().map(|pair| pair.1)
	}
}

pub struct SectorEnumerateColumns<'a, T> where T: 'a {
	sector: &'a Sector<T>,
	column: LayerPosition,
	done:   bool
}

impl<'a, T> Iterator for SectorEnumerateColumns<'a, T> where T: 'a {
	type Item = (LayerPosition, [Option<&'a T>; 16]);

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}

		let mut chunks = [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None];

		for y in 0..16 {
			let position = ChunkPosition::from_layer(y, self.column);

			chunks[y as usize] = self.sector[position].as_ref();
		}

		let position = self.column;

		if self.column == LayerPosition::from_zx(255) {
			self.done = true;
		} else {
			self.column = LayerPosition::from_zx(self.column.zx() + 1);
		}

		Some((position, chunks))
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

// TODO: Enumerate iterators
pub struct SectorEnumeratePresent<'a, T> where T: 'a {
	_sector: &'a Sector<T>,
	// TODO: ChunkMask iter
}

pub struct SectorEnumeratePresentMut<'a, T> where T: 'a {
	_sector: &'a mut Sector<T>,
	// TODO: ChunkMask iter_mut
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