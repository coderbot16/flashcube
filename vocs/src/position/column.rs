use position::ChunkPosition;
use position::LayerPosition;
use std::fmt::{Debug, Formatter, Result};

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
	
	/// Creates a new ColumnPosition from a YZX index.
	/// ### Out of bounds behavior
	/// If the index is out of bounds, it is truncated.
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
	pub fn chunk(&self) -> ChunkPosition { ChunkPosition::from_yzx(self.chunk_yzx()) }

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
	
	pub fn minus_x(&self) -> Option<ColumnPosition> {
		if self.x() != 0 {
			Some(ColumnPosition(self.0 - 0x0001))
		} else {
			None
		}
	}
	
	pub fn plus_x(&self) -> Option<ColumnPosition> {
		if self.x() != 15 {
			Some(ColumnPosition(self.0 + 0x0001))
		} else {
			None
		}
	}
	
	pub fn minus_z(&self) -> Option<ColumnPosition> {
		if self.z() != 0 {
			Some(ColumnPosition(self.0 - 0x0010))
		} else {
			None
		}
	}
	
	pub fn plus_z(&self) -> Option<ColumnPosition> {
		if self.z() != 15 {
			Some(ColumnPosition(self.0 + 0x0010))
		} else {
			None
		}
	}
	
	pub fn minus_y(&self) -> Option<ColumnPosition> {
		if self.y() > 0 {
			Some(ColumnPosition(self.0 - 0x0100))
		} else {
			None
		}
	}

	pub fn plus_y(&self) -> Option<ColumnPosition> {
		if self.y() != 255 {
			Some(ColumnPosition(self.0 + 0x0100))
		} else {
			None
		}
	}
}

impl Debug for ColumnPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "ColumnPosition {{ x: {}, y: {}, z: {}, yzx: {} }}", self.x(), self.y(), self.z(), self.yzx())
	}
}