use crate::position::CubePosition;
use crate::position::{LayerPosition, Offset, dir};
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct ColumnPosition(u16);

impl ColumnPosition {
	/// Creates a new ColumnPosition from the X, Y, and Z components.
	/// ### Out of bounds behavior
	/// If the arguments are out of bounds, then they are truncated.
	pub fn new(x: u8, y: u8, z: u8) -> Self {
		ColumnPosition(
			(      (y as u16) << 8) |
			(((z&0xF) as u16) << 4) | 
			 ((x&0xF) as u16)
		)
	}
	
	/// Creates a new ColumnPosition from the Y component and LayerPosition containing the X and Z components.
	/// Out of bounds is not possible with this function.
	pub fn from_layer(y: u8, layer: LayerPosition) -> Self {
		ColumnPosition(
			((y as u16) << 8) | (layer.zx() as u16)
		)
	}

	/// Creates a new ColumnPosition from the given ChunkPosition with an additional section_y value,
	/// which determines the column "section" (from 0-16) in which this position resides.
	pub fn from_chunk(section_y: u8, chunk: CubePosition) -> Self {
		ColumnPosition::from_yzx (
			((section_y as u16) << 12) |
			chunk.yzx()
		)
	}
	
	/// Creates a new ColumnPosition from a YZX index.
	/// Out of bounds is not possible with this function.
	pub fn from_yzx(yzx: u16) -> Self {
		ColumnPosition(yzx)
	}
	
	/// Creates a new ColumnPosition from a chunk XYZ index.
	/// This function only supports Y values from 0 to 15.
	/// ### Out of bounds behavior
	/// If the index is out of bounds, it is truncated.
	pub fn from_chunk_xyz(xyz: u16) -> Self {
		let xyz = xyz & 0xFFF; // Truncate the value if too large
		// X YZ - Start
		// YZ X - End
		ColumnPosition(((xyz & 0xF00) >> 8) | ((xyz & 0x0FF) << 4))
	}
	
	/// Returns the X component.
	pub fn x(&self) -> u8 {
		(self.0 & 0x00F) as u8
	}
	
	/// Returns the Z component.
	pub fn z(&self) -> u8 {
		((self.0 & 0x0F0) >> 4) as u8
	}
	
	/// Returns the Y component.
	pub fn y(&self) -> u8 {
		(self.0 >> 8) as u8
	}
	
	/// Returns the Y component >> 4, the chunk Y.
	pub fn chunk_y(&self) -> u8 {
		(self.0 >> 12) as u8
	}
	
	/// Returns the Y and Z components, represented as `(Y<<4) | Z`.
	pub fn yz(&self) -> u16 {
		self.0 >> 4
	}

	/// Returns the Y and Z components, with Y capped to 15, represented as `(Y<<4) | Z`.
	pub fn chunk_yz(&self) -> u8 {
		(self.chunk_yzx() >> 4) as u8
	}

	/// Returns the Y and X components, with Y capped to 15, represented as `(Y<<4) | X`.
	pub fn chunk_yx(&self) -> u8 {
		(self.chunk_y() << 4) | (self.x())
	}
	
	/// Returns the index represented as `(Z<<4) | X`.
	pub fn zx(&self) -> u8 {
		(self.0 & 255) as u8
	}

	/// Returns the chunk position.
	pub fn chunk(&self) -> CubePosition { CubePosition::from_yzx(self.chunk_yzx()) }

	/// Returns the layer position. This is equivalent to `LayerPosition::from_zx(position.zx())`.
	pub fn layer(&self) -> LayerPosition {
		LayerPosition::from_zx(self.zx())
	}

	/// Returns the layer position on the PlusX / MinusX face.
	/// This is equivalent to `LayerPosition::from_zx(position.chunk_yz())`.
	pub fn layer_yz(&self) -> LayerPosition {
		LayerPosition::from_zx(self.chunk_yz())
	}

	/// Returns the layer position on the PlusZ / MinusZ face.
	/// This is equivalent to `LayerPosition::from_zx(position.chunk_yx())`.
	pub fn layer_yx(&self) -> LayerPosition {
		LayerPosition::from_zx(self.chunk_yx())
	}
	
	/// Returns the index represented as `(Y<<8) | (Z<<4) | X`.
	pub fn yzx(&self) -> u16 {
		self.0
	}
	
	/// Returns the index represented as `(Y<<8) | (Z<<4) | X` modulo 4096, for in-chunk indices.
	pub fn chunk_yzx(&self) -> u16 {
		self.0 & 4095
	}
	
	/// Returns the index represented as `(X<<8) | (Y<<4) | Z`.
	pub fn xyz(&self) -> u16 {
		((self.x() as u16) << 8) | (self.yz() & 255)
	}
	
	/// Returns the chunk_yzx index into a nibble array. Returns in the form (index, shift).
	pub fn chunk_nibble_yzx(&self) -> (usize, i8) {
		let raw = self.chunk_yzx();
		((raw >> 1) as usize, (raw & 1) as i8 * 4)
	}
}

impl fmt::Debug for ColumnPosition {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "ColumnPosition {{ x: {}, y: {}, z: {}, yzx: {} }}", self.x(), self.y(), self.z(), self.yzx())
	}
}

impl Offset<dir::Up> for ColumnPosition {
	type Spill = LayerPosition;

	fn offset(self, _: dir::Up) -> Option<Self> {
		if self.y() < 255 {
			Some(ColumnPosition(self.0 + 0x0100))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::Up) -> Self {
		ColumnPosition::from_yzx(self.0.wrapping_add(0x0100))
	}

	fn offset_spilling(self, _: dir::Up) -> Result<Self, LayerPosition> {
		if self.y() < 255 {
			Ok(ColumnPosition(self.0 + 0x0100))
		} else {
			Err(self.layer())
		}
	}
}

impl Offset<dir::Down> for ColumnPosition {
	type Spill = LayerPosition;

	fn offset(self, _: dir::Down) -> Option<Self> {
		if self.y() > 0 {
			Some(ColumnPosition(self.0 - 0x0100))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::Down) -> Self {
		ColumnPosition::from_yzx(self.0.wrapping_sub(0x0100))
	}

	fn offset_spilling(self, _: dir::Down) -> Result<Self, LayerPosition> {
		if self.y() > 0 {
			Ok(ColumnPosition(self.0 - 0x0100))
		} else {
			Err(self.layer())
		}
	}
}

impl Offset<dir::PlusX> for ColumnPosition {
	type Spill = (); // No proper coordinate type for a 256x16 surface.

	fn offset(self, _: dir::PlusX) -> Option<Self> {
		if self.x() != 15 {
			Some(ColumnPosition(self.0 + 0x0001))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::PlusX) -> Self {
		let base = self.0 & 0xFFF0;
		let add = ((self.x() + 1) & 15) as u16;

		ColumnPosition(base | add)
	}

	fn offset_spilling(self, _: dir::PlusX) -> Result<Self, ()> {
		self.offset(dir::PlusX).ok_or(())
	}
}

impl Offset<dir::MinusX> for ColumnPosition {
	type Spill = (); // No proper coordinate type for a 256x16 surface.

	fn offset(self, _: dir::MinusX) -> Option<Self> {
		if self.x() != 0 {
			Some(ColumnPosition(self.0 - 0x0001))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::MinusX) -> Self {
		let base = self.0 & 0xFFF0;
		let add = ((self.x().wrapping_sub(1)) & 15) as u16;

		ColumnPosition(base | add)
	}

	fn offset_spilling(self, _: dir::MinusX) -> Result<Self, ()> {
		self.offset(dir::MinusX).ok_or(())
	}
}

impl Offset<dir::PlusZ> for ColumnPosition {
	type Spill = (); // No proper coordinate type for a 256x16 surface.

	fn offset(self, _: dir::PlusZ) -> Option<Self> {
		if self.z() != 15 {
			Some(ColumnPosition(self.0 + 0x0010))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::PlusZ) -> Self {
		let base = self.0 & 0xFF0F;
		let add = ((self.z() + 1) & 15) as u16;

		ColumnPosition(base | (add << 4))
	}

	fn offset_spilling(self, _: dir::PlusZ) -> Result<Self, ()> {
		self.offset(dir::PlusZ).ok_or(())
	}
}

impl Offset<dir::MinusZ> for ColumnPosition {
	type Spill = (); // No proper coordinate type for a 256x16 surface.

	fn offset(self, _: dir::MinusZ) -> Option<Self> {
		if self.z() != 0 {
			Some(ColumnPosition(self.0 - 0x0010))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::MinusZ) -> Self {
		let base = self.0 & 0xFF0F;
		let add = ((self.z().wrapping_sub(1)) & 15) as u16;

		ColumnPosition(base | (add << 4))
	}

	fn offset_spilling(self, _: dir::MinusZ) -> Result<Self, ()> {
		self.offset(dir::MinusZ).ok_or(())
	}
}