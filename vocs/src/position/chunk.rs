use std::fmt;
use position::{LayerPosition, Offset, dir};
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

	/// Returns the Y and Z components, represented as `(Z<<4) | Y`.
	pub fn zy(&self) -> u8 {
		(
			 (self.0 & 0x0F0) |     // Z component, not shifted
			((self.0 & 0xF00) >> 8) // Y component, shifted to the other side
		) as u8
	}

	/// Returns the index represented as `(Z<<4) | X`.
	pub fn zx(&self) -> u8 {
		(self.0 & 0x0FF) as u8
	}

	/// Returns the Y and X components, represented as `(Y<<4) | X`.
	pub fn yx(&self) -> u8 {
		(
			((self.0 & 0xF00) >> 4) | // Y component, shifted by one space
			 (self.0 & 0x00F)         // X component, not shifted
		) as u8
	}

	// Layer swizzle functions

	/// Returns the layer position. This is equivalent to `LayerPosition::from_zx(position.zx())`.
	pub fn layer(&self) -> LayerPosition {
		LayerPosition::from_zx(self.zx())
	}

	/// Returns the layer position on the PlusX / MinusX face.
	/// This is equivalent to `LayerPosition::from_zx(position.zy())`.
	pub fn layer_zy(&self) -> LayerPosition {
		LayerPosition::from_zx(self.zy())
	}

	/// Returns the layer position on the PlusZ / MinusZ face.
	/// This is equivalent to `LayerPosition::from_zx(position.yx())`.
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
		((self.x() as u16) << 8) | ((self.y() as u16) << 4) |(self.z() as u16)
	}

	/// Returns the chunk_yzx index into a nibble array. Returns in the form (index, shift).
	pub fn yzx_nibble(&self) -> (usize, u8) {
		let raw = self.yzx();
		((raw >> 1) as usize, (raw & 1) as u8 * 4)
	}

	// Individual component setting

	/// Replaces the X component with the specified value, leaving Y and Z the same.
	pub fn with_x(&self, x: u8) -> Self {
		let x = x as u16;

		ChunkPosition((self.0 & 0x0FF0) | (x & 0x000F))
	}

	/// Replaces the Y component with the specified value, leaving X and Z the same.
	pub fn with_y(&self, y: u8) -> Self {
		let y = y as u16;

		ChunkPosition((self.0 & 0x00FF) | ((y & 0x000F) << 8))
	}

	/// Replaces the Z component with the specified value, leaving X and Y the same.
	pub fn with_z(&self, z: u8) -> Self {
		let z = z as u16;

		ChunkPosition((self.0 & 0x0F0F) | ((z & 0x000F) << 4))
	}

	// Iteration

	pub fn enumerate() -> Enumerate {
		Enumerate { index: 0 }
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

impl fmt::Display for ChunkPosition {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
	}
}

impl fmt::Debug for ChunkPosition {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "ChunkPosition {{ x: {}, y: {}, z: {} }}", self.x(), self.y(), self.z())
	}
}

impl Offset<dir::Up> for ChunkPosition {
	type Spill = LayerPosition;

	fn offset(self, _: dir::Up) -> Option<Self> {
		if self.y() < 15 {
			Some(ChunkPosition(self.0 + 0x0100))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::Up) -> Self {
		ChunkPosition::from_yzx(self.0 + 0x0100)
	}

	fn offset_spilling(self, _: dir::Up) -> Result<Self, LayerPosition> {
		if self.y() < 15 {
			Ok(ChunkPosition(self.0 + 0x0100))
		} else {
			Err(self.layer())
		}
	}
}

impl Offset<dir::Down> for ChunkPosition {
	type Spill = LayerPosition;

	fn offset(self, _: dir::Down) -> Option<Self> {
		if self.y() > 0 {
			Some(ChunkPosition(self.0 - 0x0100))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::Down) -> Self {
		ChunkPosition::from_yzx(self.0.wrapping_sub(0x0100))
	}

	fn offset_spilling(self, _: dir::Down) -> Result<Self, LayerPosition> {
		if self.y() > 0 {
			Ok(ChunkPosition(self.0 - 0x0100))
		} else {
			Err(self.layer())
		}
	}
}

impl Offset<dir::PlusX> for ChunkPosition {
	// X coordinate is erased, leaving the zy coordinates.
	type Spill = LayerPosition;

	fn offset(self, _: dir::PlusX) -> Option<Self> {
		if self.x() != 15 {
			Some(ChunkPosition(self.0 + 0x0001))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::PlusX) -> Self {
		let base = self.0 & 0x0FF0;
		let add = ((self.x() + 1) & 15) as u16;

		ChunkPosition(base | add)
	}

	fn offset_spilling(self, _: dir::PlusX) -> Result<Self, LayerPosition> {
		if self.x() != 15 {
			Ok(ChunkPosition(self.0 + 0x0001))
		} else {
			Err(self.layer_zy())
		}
	}
}

impl Offset<dir::MinusX> for ChunkPosition {
	// X coordinate is erased, leaving the zy coordinates.
	type Spill = LayerPosition;

	fn offset(self, _: dir::MinusX) -> Option<Self> {
		if self.x() != 0 {
			Some(ChunkPosition(self.0 - 0x0001))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::MinusX) -> Self {
		let base = self.0 & 0x0FF0;
		let add = ((self.x().wrapping_sub(1)) & 15) as u16;

		ChunkPosition(base | add)
	}

	fn offset_spilling(self, _: dir::MinusX) -> Result<Self, LayerPosition> {
		if self.x() != 0 {
			Ok(ChunkPosition(self.0 - 0x0001))
		} else {
			Err(self.layer_zy())
		}
	}
}

impl Offset<dir::PlusZ> for ChunkPosition {
	// Z coordinate is erased, leaving the yx coordinates.
	type Spill = LayerPosition;

	fn offset(self, _: dir::PlusZ) -> Option<Self> {
		if self.z() != 15 {
			Some(ChunkPosition(self.0 + 0x0010))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::PlusZ) -> Self {
		let base = self.0 & 0x0F0F;
		let add = ((self.z() + 1) & 15) as u16;

		ChunkPosition(base | (add << 4))
	}

	fn offset_spilling(self, _: dir::PlusZ) -> Result<Self, LayerPosition> {
		if self.z() != 15 {
			Ok(ChunkPosition(self.0 + 0x0010))
		} else {
			Err(self.layer_yx())
		}
	}
}

impl Offset<dir::MinusZ> for ChunkPosition {
	// Z coordinate is erased, leaving the yx coordinates.
	type Spill = LayerPosition;

	fn offset(self, _: dir::MinusZ) -> Option<Self> {
		if self.z() != 0 {
			Some(ChunkPosition(self.0 - 0x0010))
		} else {
			None
		}
	}

	fn offset_wrapping(self, _: dir::MinusZ) -> Self {
		let base = self.0 & 0x0F0F;
		let add = ((self.z().wrapping_sub(1)) & 15) as u16;

		ChunkPosition(base | (add << 4))
	}

	fn offset_spilling(self, _: dir::MinusZ) -> Result<Self, LayerPosition> {
		if self.z() != 0 {
			Ok(ChunkPosition(self.0 - 0x0010))
		} else {
			Err(self.layer_yx())
		}
	}
}

pub struct Enumerate {
	index: u16
}

impl Iterator for Enumerate {
	type Item = ChunkPosition;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < 4096 {
			let position = ChunkPosition::from_yzx(self.index);

			self.index += 1;

			Some(position)
		} else {
			None
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_with() {
		let position = ChunkPosition::new(6, 6, 6);
		assert_eq!(position.with_x(9), ChunkPosition::new(9, 6, 6));
		assert_eq!(position.with_y(9), ChunkPosition::new(6, 9, 6));
		assert_eq!(position.with_z(9), ChunkPosition::new(6, 6, 9));
	}
}