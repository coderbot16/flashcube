use bit_vec::BitVec;
use storage::mask::Mask;
use std::marker::PhantomData;

pub struct Pool<P, T> where T: Mask<P> + Default {
	pub pool: Vec<(T, u16)>, // TODO TODO TODO
	free: BitVec,
	first_free: Option<usize>,
	max_size: usize,
	phantom: PhantomData<P>
}

impl<P, T> Pool<P, T> where T: Mask<P> + Default + Clone {
	pub fn new(start_size: usize, max_size: usize) -> Self {
		assert!(start_size <= max_size, "Max size greater than start size for Pool");

		Pool {
			pool: vec![(T::default(), 0); start_size],
			free: BitVec::from_elem(start_size, true),
			first_free: if start_size > 0 { Some(0) } else { None },
			max_size,
			phantom: PhantomData
		}
	}
}

impl<P, T> Pool<P, T> where T: Mask<P> + Default {
	pub fn empty(max_size: usize) -> Self {
		Pool {
			pool: Vec::new(),
			free: BitVec::new(),
			first_free: None,
			max_size,
			phantom: PhantomData
		}
	}

	pub fn alloc(&mut self) -> Option<usize> {
		if let Some(free) = self.first_free {
			self.free.set(free, false);
			self.first_free = self.free.scan_clear().into_iter().next();

			Some(free)
		} else if self.pool.len() < self.max_size {
			let index = self.pool.len();

			self.free.push(false);
			self.pool.push((T::default(), 0));

			Some(index)
		} else {
			None
		}
	}

	pub fn free(&mut self, index: usize) {
		if let Some(ref mut free) = self.first_free {
			if index < *free {
				*free = index;
			}
		} else {
			self.first_free = Some(index);
		}

		let entry = &mut self.pool[index];
		entry.0.clear();
		entry.1 = 0;

		self.free.set(index, true);
	}

	pub fn clear(&mut self) {
		self.free.clear();

		if self.pool.len() != 0 {
			self.first_free = Some(0);
		}
	}
}