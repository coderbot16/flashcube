use std::ops::{Add, AddAssign};
use vocs::position::LayerPosition;

pub struct Layer<T>([T; 256]) where T: Copy;
impl<T> Layer<T> where T: Copy {
	pub fn fill(fill: T) -> Self {
		Layer([fill; 256])
	}

	pub fn get(&self, position: LayerPosition) -> T {
		self.0[position.zx() as usize]
	}

	pub fn set(&mut self, position: LayerPosition, value: T) {
		self.0[position.zx() as usize] = value;
	}
}

impl<T> Add for Layer<T> where T: Copy + AddAssign {
	type Output = Self;

	fn add(mut self, rhs: Self) -> Self::Output {
		for x in 0..256 {
			// TODO: Iterators.
			self.0[x] += rhs.0[x];
		}

		self
	}
}

impl<T> AddAssign for Layer<T> where T: Copy + AddAssign {
	fn add_assign(&mut self, rhs: Self) {
		for x in 0..256 {
			// TODO: Iterators.
			self.0[x] += rhs.0[x];
		}
	}
}