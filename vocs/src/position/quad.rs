use position::ColumnPosition;
use std::fmt::{Debug, Formatter, Result};

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

	/// Offsets this position by the coordinates.
	/// Returns None if they would be out of bounds.
	pub fn offset(&self, x: i8, y: i8, z: i8) -> Option<Self> {
		let x = (self.x() as i16) + (x as i16);
		let y = (self.y() as i16) + (y as i16);
		let z = (self.z() as i16) + (z as i16);

		if x > 31 || y > 255 || z > 31 || x < 0 || y < 0 || z < 0 {
			None
		} else {
			Some(QuadPosition::new(x as u8, y as u8, z as u8))
		}
	}

	pub fn minus_y(&self) -> Option<QuadPosition> { self.0.minus_y().map(|c| QuadPosition(c, self.1)) }
	pub fn plus_y(&self)  -> Option<QuadPosition> { self.0.plus_y() .map(|c| QuadPosition(c, self.1)) }

	// TODO: Is there a more efficient way to implement these?
	pub fn minus_x(&self) -> Option<QuadPosition> { self.offset(-1, 0 , 0) }
	pub fn plus_x(&self)  -> Option<QuadPosition> { self.offset( 1, 0 , 0) }
	pub fn minus_z(&self) -> Option<QuadPosition> { self.offset(0, 0 , -1) }
	pub fn plus_z(&self)  -> Option<QuadPosition> { self.offset( 0, 0 , 1) }
}

impl Debug for QuadPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "QuadPosition {{ x: {}, y: {}, z: {} }}", self.x(), self.y(), self.z())
	}
}