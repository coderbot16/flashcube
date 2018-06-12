use position::{ColumnPosition, Offset, dir};
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct QuadPosition(ColumnPosition, u8);

impl QuadPosition {
	/// Creates a new QuadPosition from the X, Y, and Z components.
	/// ### Out of bounds behavior
	/// If the arguments are out of bounds, then they are truncated.
	pub fn new(x: u8, y: u8, z: u8) -> Self {
		let q = ((x >> 4) & 1) | ((z >> 3) & 2);

		QuadPosition(ColumnPosition::new(x, y, z), q)
	}

	/// Creates a new QuadPosition from a ColumnPosition, relative to the center part of the quad.
	/// This is equivalent to QuadPosition::new(column.x() + 8, column.y(), column.z() + 8).
	pub fn from_centered(column: ColumnPosition) -> Self {
		QuadPosition::new(column.x() + 8, column.y(), column.z() + 8)
	}

	/// Returns the X component.
	pub fn x(&self) -> u8 {
		self.0.x() | ((self.1 & 1) << 4)
	}

	/// Returns the Z component.
	pub fn z(&self) -> u8 {
		self.0.z() | ((self.1 & 2) << 3)
	}

	/// Returns the Y component.
	pub fn y(&self) -> u8 {
		self.0.y()
	}

	/// Returns the column position.
	pub fn column(&self) -> ColumnPosition { self.0 }

	/// Returns the index of the column, from 0 to 3.
	pub fn q(&self) -> u8 { self.1 }

	/// The opposite of from_centered.
	/// Returns an Option because not all QuadPositions are in the center of the quad.
	pub fn to_centered(&self) -> Option<ColumnPosition> {
		let (x, z) = (self.x(), self.z());

		if x < 8 || x > 23 || z < 8 || z > 23 {
			None
		} else {
			Some(ColumnPosition::new(x - 8, self.y(), z - 8))
		}
	}
}

impl fmt::Debug for QuadPosition {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "QuadPosition {{ x: {}, y: {}, z: {} }}", self.x(), self.y(), self.z())
	}
}

impl Offset<(i8, i8, i8)> for QuadPosition {
	type Spill = ();

	fn offset(self, (x, y, z): (i8, i8, i8)) -> Option<Self> {
		let x = (self.x() as i16) + (x as i16);
		let y = (self.y() as i16) + (y as i16);
		let z = (self.z() as i16) + (z as i16);

		if x > 31 || y > 255 || z > 31 || x < 0 || y < 0 || z < 0 {
			None
		} else {
			Some(QuadPosition::new(x as u8, y as u8, z as u8))
		}
	}

	fn offset_wrapping(self, (x, y, z): (i8, i8, i8)) -> Self {
		let x = (self.x() as i16) + (x as i16);
		let y = (self.y() as i16) + (y as i16);
		let z = (self.z() as i16) + (z as i16);

		let (x, y, z) = (
			(x as u8) & 31,
			(y as u8),
			(z as u8) & 31
		);

		QuadPosition::new(x, y, z)
	}

	fn offset_spilling(self, offs: (i8, i8, i8)) -> Result<Self, ()> {
		self.offset(offs).ok_or(())
	}
}

impl Offset<dir::Up> for QuadPosition {
	type Spill = ();

	fn offset(self, _: dir::Up) -> Option<Self> {
		self.0.offset(dir::Up).map(|c| QuadPosition(c, self.1))
	}

	fn offset_wrapping(self, _: dir::Up) -> Self {
		let c = self.0.offset_wrapping(dir::Up);

		QuadPosition(c, self.1)
	}

	fn offset_spilling(self, _: dir::Up) -> Result<Self, ()> {
		self.offset(dir::Up).ok_or(())
	}
}

impl Offset<dir::Down> for QuadPosition {
	type Spill = ();

	fn offset(self, _: dir::Down) -> Option<Self> {
		self.0.offset(dir::Down).map(|c| QuadPosition(c, self.1))
	}

	fn offset_wrapping(self, _: dir::Down) -> Self {
		let c = self.0.offset_wrapping(dir::Down);

		QuadPosition(c, self.1)
	}

	fn offset_spilling(self, _: dir::Down) -> Result<Self, ()> {
		self.offset(dir::Down).ok_or(())
	}
}

// TODO: Is there a more efficient way to implement these?

impl Offset<dir::PlusX> for QuadPosition {
	type Spill = ();

	fn offset(self, _: dir::PlusX) -> Option<Self> {
		self.offset((1, 0 , 0))
	}

	fn offset_wrapping(self, _: dir::PlusX) -> Self {
		self.offset_wrapping((1, 0, 0))
	}

	fn offset_spilling(self, _: dir::PlusX) -> Result<Self, ()> {
		self.offset_spilling((1, 0 , 0))
	}
}

impl Offset<dir::MinusX> for QuadPosition {
	type Spill = ();

	fn offset(self, _: dir::MinusX) -> Option<Self> {
		self.offset((-1, 0, 0))
	}

	fn offset_wrapping(self, _: dir::MinusX) -> Self {
		self.offset_wrapping((-1, 0, 0))
	}

	fn offset_spilling(self, _: dir::MinusX) -> Result<Self, ()> {
		self.offset_spilling((-1, 0 , 0))
	}
}

impl Offset<dir::PlusZ> for QuadPosition {
	type Spill = ();

	fn offset(self, _: dir::PlusZ) -> Option<Self> {
		self.offset((0, 0, 1))
	}

	fn offset_wrapping(self, _: dir::PlusZ) -> Self {
		self.offset_wrapping((0, 0, 1))
	}

	fn offset_spilling(self, _: dir::PlusZ) -> Result<Self, ()> {
		self.offset_spilling((0, 0, 1))
	}
}

impl Offset<dir::MinusZ> for QuadPosition {
	type Spill = ();

	fn offset(self, _: dir::MinusZ) -> Option<Self> {
		self.offset((0, 0, -1))
	}

	fn offset_wrapping(self, _: dir::MinusZ) -> Self {
		self.offset_wrapping((0, 0, -1))
	}

	fn offset_spilling(self, _: dir::MinusZ) -> Result<Self, ()> {
		self.offset_spilling((0, 0, -1))
	}
}