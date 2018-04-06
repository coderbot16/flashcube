use packed::PackedIndex;

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
	/// ### Out of bounds behavior
	/// If the index is out of bounds, it is truncated.
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
}

impl PackedIndex for LayerPosition {
	fn size_factor() -> usize {
		4
	}

	fn from_usize(index: usize) -> Self {
		LayerPosition::from_zx(index as u8)
	}

	fn to_usize(&self) -> usize {
		self.zx() as usize
	}
}