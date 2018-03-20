use std::fmt::{Debug, Display, Result, Formatter};
use position::LayerPosition;
use storage::packed::PackedIndex;

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct ChunkPosition(u16);

impl ChunkPosition {
	/// Creates a new ChunkPosition from the X, Y, and Z components.
	/// ### Out of bounds behavior
	/// If the arguments are out of bounds, then they are truncated.
	pub fn new(x: u8, y: u8, z: u8) -> Self {
		ChunkPosition(
			(((y&0xF) as u16) << 8) |
			(((z&0xF) as u16) << 4) |
			 ((x&0xF) as u16)
		)
	}

	/// Creates a new ChunkPosition from the Y component and LayerPosition containing the X and Z components.
	/// Out of bounds is not possible with this function.
	pub fn from_layer(y: u8, layer: LayerPosition) -> Self {
		ChunkPosition(
			(((y&0xF) as u16) << 8) | (layer.zx() as u16)
		)
	}

	/// Creates a new ChunkPosition from a YZX index.
	/// ### Out of bounds behavior
	/// If the index is out of bounds, it is truncated.
	pub fn from_yzx(yzx: u16) -> Self {
		ChunkPosition(yzx % 4096)
	}

	/// Creates a new ChunkPosition from a XYZ index.
	/// ### Out of bounds behavior
	/// If the index is out of bounds, it is truncated.
	pub fn from_xyz(xyz: u16) -> Self {
		let xyz = xyz & 0xFFF; // Truncate the value if too large
		// X YZ - Start
		// YZ X - End
		ChunkPosition(((xyz & 0xF00) >> 8) | ((xyz & 0x0FF) << 4))
	}

	// Component access

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
		((self.0 & 0xF00) >> 8) as u8
	}

	// Swizzle functions

	/// Returns the Y and Z components, represented as `(Y<<4) | Z`.
	pub fn yz(&self) -> u8 {
		(self.0 >> 4) as u8
	}

	/// Returns the index represented as `(Z<<4) | X`.
	pub fn zx(&self) -> u8 {
		(self.0 & 255) as u8
	}

	/// Returns the Y and X components, represented as `(Y<<4) | X`.
	pub fn yx(&self) -> u8 {
		(self.y() << 4) | (self.x())
	}

	// Layer swizzle functions

	/// Returns the layer position. This is equivalent to `LayerPosition::from_zx(position.zx())`.
	pub fn layer(&self) -> LayerPosition {
		LayerPosition::from_zx(self.zx())
	}

	/// Returns the layer position on the PlusX / MinusX face.
	/// This is equivalent to `LayerPosition::from_zx(position.chunk_yz())`.
	pub fn layer_yz(&self) -> LayerPosition {
		LayerPosition::from_zx(self.yz())
	}

	/// Returns the layer position on the PlusZ / MinusZ face.
	/// This is equivalent to `LayerPosition::from_zx(position.chunk_yx())`.
	pub fn layer_yx(&self) -> LayerPosition {
		LayerPosition::from_zx(self.yx())
	}

	// Full component access and swizzling

	/// Returns the index represented as `(Y<<8) | (Z<<4) | X` modulo 4096, for in-chunk indices.
	pub fn yzx(&self) -> u16 {
		self.0 & 4095
	}

	/// Returns the index represented as `(X<<8) | (Y<<4) | Z`.
	pub fn xyz(&self) -> u16 {
		((self.x() as u16) << 8) | ((self.yz() as u16) & 255)
	}

	/// Returns the chunk_yzx index into a nibble array. Returns in the form (index, shift).
	pub fn yzx_nibble(&self) -> (usize, u8) {
		let raw = self.yzx();
		((raw >> 1) as usize, (raw & 1) as u8 * 4)
	}

	// Component offsetting

	/// Subtracts 1 from X, returning None if it would underflow.
	pub fn minus_x(&self) -> Option<ChunkPosition> {
		if self.x() != 0 {
			Some(ChunkPosition(self.0 - 0x0001))
		} else {
			None
		}
	}


	/// Adds 1 to X, returning None if it would overflow.
	pub fn plus_x(&self) -> Option<ChunkPosition> {
		if self.x() != 15 {
			Some(ChunkPosition(self.0 + 0x0001))
		} else {
			None
		}
	}

	/// Subtracts 1 from Z, returning None if it would underflow.
	pub fn minus_z(&self) -> Option<ChunkPosition> {
		if self.z() != 0 {
			Some(ChunkPosition(self.0 - 0x0010))
		} else {
			None
		}
	}

	/// Adds 1 to Z, returning None if it would overflow.
	pub fn plus_z(&self) -> Option<ChunkPosition> {
		if self.z() != 15 {
			Some(ChunkPosition(self.0 + 0x0010))
		} else {
			None
		}
	}

	/// Subtracts 1 from Y, returning None if it would underflow.
	pub fn minus_y(&self) -> Option<ChunkPosition> {
		if self.y() > 0 {
			Some(ChunkPosition(self.0 - 0x0100))
		} else {
			None
		}
	}

	/// Adds 1 to Y, returning None if it would overflow.
	pub fn plus_y(&self) -> Option<ChunkPosition> {
		if self.y() < 15 {
			Some(ChunkPosition(self.0 + 0x0100))
		} else {
			None
		}
	}
}

impl PackedIndex for ChunkPosition {
	fn entries() -> usize {
		4096
	}

	fn from_index(index: usize) -> Self {
		debug_assert!(index < 4096);

		ChunkPosition::from_yzx(index as u16)
	}

	fn to_index(&self) -> usize {
		self.yzx() as usize
	}
}

impl Display for ChunkPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
	}
}

impl Debug for ChunkPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "ChunkPosition {{ x: {}, y: {}, z: {} }}", self.x(), self.y(), self.z())
	}
}