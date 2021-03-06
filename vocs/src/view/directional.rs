use crate::position::{Dir, dir};
use std::ops::{Index, IndexMut};

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct SplitDirectional<T> {
	pub plus_x:  T,
	pub minus_x: T,
	pub up:      T,
	pub down:    T,
	pub plus_z:  T,
	pub minus_z: T
}

impl<T: Copy> SplitDirectional<T> {
	pub fn splat(value: T) -> Self {
		SplitDirectional {
			plus_x: value,
			minus_x: value,
			up: value,
			down: value,
			plus_z: value,
			minus_z: value
		}
	}
}

impl<T> SplitDirectional<T> {
	pub fn as_ref<'a>(&'a self) -> SplitDirectional<&'a T> {
		SplitDirectional {
			plus_x: &self.plus_x,
			minus_x: &self.minus_x,
			up: &self.up,
			down: &self.down,
			plus_z: &self.plus_z,
			minus_z: &self.minus_z
		}
	}

	pub fn map<F, M>(self, f: F) -> SplitDirectional<M> where F: Fn(T) -> M {
		SplitDirectional {
			plus_x: f(self.plus_x),
			minus_x: f(self.minus_x),
			up: f(self.up),
			down: f(self.down),
			plus_z: f(self.plus_z),
			minus_z: f(self.minus_z)
		}
	}
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Directional<T>([T; 6]);

impl<T: Copy> Directional<T> {
	pub fn splat(value: T) -> Self {
		Directional([value; 6])
	}
}

impl<T> Directional<T> {
	pub fn combine(split: SplitDirectional<T>) -> Self {
		Directional ([
			split.plus_x,
			split.minus_x,
			split.up,
			split.down,
			split.plus_z,
			split.minus_z
		])
	}
	
	pub fn split(self) -> SplitDirectional<T> {
		let [plus_x, minus_x, up, down, plus_z, minus_z] = self.0;

		SplitDirectional {
			plus_x,
			minus_x,
			up,
			down,
			plus_z,
			minus_z
		}
	}
}

impl<T> Index<Dir> for Directional<T> {
	type Output = T;

	fn index(&self, direction: Dir) -> &T {
		&self.0[direction as usize]
	}
}

impl<T> IndexMut<Dir> for Directional<T> {
	fn index_mut(&mut self, direction: Dir) -> &mut T {
		&mut self.0[direction as usize]
	}
}

impl<T> Index<dir::Up> for Directional<T> {
	type Output = T;

	fn index(&self, _: dir::Up) -> &T {
		&self.0[Dir::Up as usize]
	}
}

impl<T> IndexMut<dir::Up> for Directional<T> {
	fn index_mut(&mut self, _: dir::Up) -> &mut T {
		&mut self.0[Dir::Up as usize]
	}
}

impl<T> Index<dir::Down> for Directional<T> {
	type Output = T;

	fn index(&self, _: dir::Down) -> &T {
		&self.0[Dir::Down as usize]
	}
}

impl<T> IndexMut<dir::Down> for Directional<T> {
	fn index_mut(&mut self, _: dir::Down) -> &mut T {
		&mut self.0[Dir::Down as usize]
	}
}

impl<T> Index<dir::PlusX> for Directional<T> {
	type Output = T;

	fn index(&self, _: dir::PlusX) -> &T {
		&self.0[Dir::PlusX as usize]
	}
}

impl<T> IndexMut<dir::PlusX> for Directional<T> {
	fn index_mut(&mut self, _: dir::PlusX) -> &mut T {
		&mut self.0[Dir::PlusX as usize]
	}
}

impl<T> Index<dir::MinusX> for Directional<T> {
	type Output = T;

	fn index(&self, _: dir::MinusX) -> &T {
		&self.0[Dir::MinusX as usize]
	}
}

impl<T> IndexMut<dir::MinusX> for Directional<T> {
	fn index_mut(&mut self, _: dir::MinusX) -> &mut T {
		&mut self.0[Dir::MinusX as usize]
	}
}

impl<T> Index<dir::PlusZ> for Directional<T> {
	type Output = T;

	fn index(&self, _: dir::PlusZ) -> &T {
		&self.0[Dir::PlusZ as usize]
	}
}

impl<T> IndexMut<dir::PlusZ> for Directional<T> {
	fn index_mut(&mut self, _: dir::PlusZ) -> &mut T {
		&mut self.0[Dir::PlusZ as usize]
	}
}

impl<T> Index<dir::MinusZ> for Directional<T> {
	type Output = T;

	fn index(&self, _: dir::MinusZ) -> &T {
		&self.0[Dir::MinusZ as usize]
	}
}

impl<T> IndexMut<dir::MinusZ> for Directional<T> {
	fn index_mut(&mut self, _: dir::MinusZ) -> &mut T {
		&mut self.0[Dir::MinusZ as usize]
	}
}

// TODO: Axial