use std::fmt::{Debug, Display, Result, Formatter};
use position::{LayerPosition, Offset, Up, Down, PlusX, MinusX, PlusZ, MinusZ};
use packed::PackedIndex;

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct ChunkPosition(u16);

impl ChunkPosition {
	/// Creates a new ChunkPosition from the X, Y, and Z components.
	/// ### Out of bounds behavior
	/// If the arguments are out of bounds, then they are truncated.
	pub fn new(x: u8, y: u8, z: u8) -> Self {
		ChunkPosition (
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
}

impl PackedIndex for ChunkPosition {
	fn size_factor() -> usize {
		64
	}

	fn from_usize(index: usize) -> Self {
		debug_assert!(index < 4096);

		ChunkPosition::from_yzx(index as u16)
	}

	fn to_usize(&self) -> usize {
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

impl Offset<Up> for ChunkPosition {
	fn offset(self, _: Up) -> Option<Self> {
		if self.y() < 15 {
			Some(ChunkPosition(self.0 + 0x0100))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: Up) -> Self {
		ChunkPosition::from_yzx(self.0 + 0x0100)
	}
}

impl Offset<Down> for ChunkPosition {
	fn offset(self, _: Down) -> Option<Self> {
		if self.y() > 0 {
			Some(ChunkPosition(self.0 - 0x0100))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: Down) -> Self {
		ChunkPosition::from_yzx(self.0.wrapping_sub(0x0100))
	}
}

impl Offset<PlusX> for ChunkPosition {
	fn offset(self, _: PlusX) -> Option<Self> {
		if self.x() != 15 {
			Some(ChunkPosition(self.0 + 0x0001))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: PlusX) -> Self {
		let base = self.0 & 0x0FF0;
		let add = ((self.x() + 1) & 15) as u16;

		ChunkPosition(base | add)
	}
}

impl Offset<MinusX> for ChunkPosition {
	fn offset(self, _: MinusX) -> Option<Self> {
		if self.x() != 0 {
			Some(ChunkPosition(self.0 - 0x0001))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: MinusX) -> Self {
		let base = self.0 & 0x0FF0;
		let add = ((self.x().wrapping_sub(1)) & 15) as u16;

		ChunkPosition(base | add)
	}
}

impl Offset<PlusZ> for ChunkPosition {
	fn offset(self, _: PlusZ) -> Option<Self> {
		if self.z() != 15 {
			Some(ChunkPosition(self.0 + 0x0010))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: PlusZ) -> Self {
		let base = self.0 & 0x0F0F;
		let add = ((self.z() + 1) & 15) as u16;

		ChunkPosition(base | (add << 4))
	}
}

impl Offset<MinusZ> for ChunkPosition {
	fn offset(self, _: MinusZ) -> Option<Self> {
		if self.z() != 0 {
			Some(ChunkPosition(self.0 - 0x0010))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: MinusZ) -> Self {
		let base = self.0 & 0x0F0F;
		let add = ((self.z().wrapping_sub(1)) & 15) as u16;

		ChunkPosition(base | (add << 4))
	}
}