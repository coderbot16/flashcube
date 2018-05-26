use position::{Dir, dir};
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

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Directional<T>([T; 6]);

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
		/*let [plus_x, minus_x, up, down, plus_z, minus_z] = self.0;

		SplitDirectional {
			plus_x,
			minus_x,
			up,
			down,
			plus_z,
			minus_z
		}*/

		// TODO: doesn't compile for some reason?
		unimplemented!()
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