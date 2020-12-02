pub trait Recycler<T> where T: Default {
	fn create(&mut self) -> T;
	fn destroy(&mut self, value: T);
}

// TODO: Unused
/*/// Stub recycler. Never retains any storage.
pub struct Trash;
impl<T> Recycler<T> for Trash where T: Default {
	fn create(&mut self) -> T {
		T::default()
	}

	fn destroy(&mut self, value: T) {
		// Explicit
		::std::mem::drop(value)
	}
}*/

/// Retains a maximum number of elements at a time.
pub struct AllocCache<T> where T: Default {
	available: Vec<T>,
	max: usize
}

impl<T> AllocCache<T> where T: Default {
	pub fn new(max: usize) -> Self {
		AllocCache {
			available: Vec::with_capacity(max), // TODO: cap initial allocation
			max
		}
	}

	// TODO: Unused
	/*pub fn remaining_capacity(&self) -> usize {
		self.max - self.available.len()
	}*/
}

impl<T> Recycler<T> for AllocCache<T> where T: Default {
	fn create(&mut self) -> T {
		self.available.pop().unwrap_or_else(Default::default)
	}

	fn destroy(&mut self, value: T) {
		if self.available.len() < self.max {
			self.available.push(value);
		} else {
			// Explicit
			::std::mem::drop(value)
		}
	}
}