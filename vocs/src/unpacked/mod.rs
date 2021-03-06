use std::ops::{Add, AddAssign, Index, IndexMut};
use std::iter::FromIterator;
use crate::position::LayerPosition;

pub struct Layer<T>(Box<[T]>);

impl<T> Layer<T> {
	pub fn map<F, V>(self, mapper: F) -> Layer<V> where F: FnMut(T) -> V {
		let entries: Vec<V> = self.0.into_vec().into_iter().map(mapper).collect();

		Layer(entries.into_boxed_slice())
	}

	pub fn into_inner(self) -> Box<[T]> {
		self.0
	}
}

impl<T> Layer<T> where T: Clone {
	pub fn filled(value: T) -> Self {
		Layer(vec![value; 256].into_boxed_slice())
	}
}

impl<T> Default for Layer<T> where T: Default {
	fn default() -> Self {
		let values: Vec<T> = (0..256).map(|_| T::default()).collect();

		Layer(values.into_boxed_slice())
	}
}

impl<T> Index<LayerPosition> for Layer<T> {	
	type Output = T;

	fn index(&self, index: LayerPosition) -> &Self::Output {
		&self.0[index.zx() as usize]
	}
}

impl<T> IndexMut<LayerPosition> for Layer<T> {
	fn index_mut(&mut self, index: LayerPosition) -> &mut Self::Output {
		&mut self.0[index.zx() as usize]
	}
}

impl<T> Add for Layer<T>
where
	T: Copy + AddAssign,
{
	type Output = Self;

	fn add(mut self, rhs: Self) -> Self::Output {
		for x in 0..256 {
			// TODO: Iterators.
			self.0[x] += rhs.0[x];
		}

		self
	}
}

impl<T> AddAssign for Layer<T>
where
	T: Copy + AddAssign,
{
	fn add_assign(&mut self, rhs: Self) {
		for x in 0..256 {
			// TODO: Iterators.
			self.0[x] += rhs.0[x];
		}
	}
}

// TODO: Implement FromParallelIterator

impl<T> FromIterator<(LayerPosition, T)> for Layer<Option<T>> {
	fn from_iter<I: IntoIterator<Item = (LayerPosition, T)>>(iter: I) -> Self {
		let mut target: Layer<Option<T>> = Layer::default();

		for (position, value) in iter {
			target[position] = Some(value);
		}

		target
	}
}
