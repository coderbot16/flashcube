use crate::packed::PackedIndex;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LayerPosition(u8);

impl LayerPosition {
	/// Creates a new LayerPosition from the X and Z components.
	/// ### Out of bounds behavior
	/// If the arguments are out of bounds, then they are truncated.
	pub fn new(x: u8, z: u8) -> Self {
		LayerPosition(((z&0xF) << 4) | (x&0xF))
	}

	/// Creates a new LayerPosition from a ZX index.
	/// Out of bounds is not possible with this function.
	pub fn from_zx(zx: u8) -> Self {
		LayerPosition(zx)
	}

	/// Returns the X component.
	pub fn x(&self) -> u8 {
		self.0 & 0x0F
	}

	/// Returns the Z component.
	pub fn z(&self) -> u8 {
		self.0 >> 4
	}

	/// Returns the index represented as `(Z<<4) | X`.
	pub fn zx(&self) -> u8 {
		self.0
	}

	// Individual component setting

	/// Replaces the X component with the specified value, leaving Y the same.
	pub fn with_x(&self, x: u8) -> Self {
		LayerPosition((self.0 & 0xF0) | (x & 0x0F))
	}

	/// Replaces the Z component with the specified value, leaving X the same.
	pub fn with_z(&self, y: u8) -> Self {
		LayerPosition((self.0 & 0x0F) | ((y & 0x0F) << 4))
	}

	// Iteration

	pub fn enumerate() -> Enumerate {
		Enumerate { index: 0 }
	}
}

impl PackedIndex for LayerPosition {
	type Enumerate = Enumerate;

	fn size_factor() -> usize {
		4
	}

	fn from_usize(index: usize) -> Self {
		LayerPosition::from_zx(index as u8)
	}

	fn to_usize(&self) -> usize {
		self.zx() as usize
	}

	fn enumerate() -> Self::Enumerate {
		Self::enumerate()
	}
}

pub struct Enumerate {
	index: u16
}

impl Iterator for Enumerate {
	type Item = LayerPosition;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < 256 {
			let position = LayerPosition::from_zx(self.index as u8);

			self.index += 1;

			Some(position)
		} else {
			None
		}
	}
}