use crate::position::{CubePosition, LayerPosition};
use std::fmt::{Debug, Display, Result, Formatter};

const MAX_U56: u64 =  72057594037927935;
const MAX_U28: u64 =  268435455;

// Note: Due to alignment, this will still be 12 bytes as if it were an (i32, i32, i32).
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct GlobalPosition {
	x: i32,
	z: i32,
	y: u8
}

impl GlobalPosition {
	pub fn new(x: i32, y: u8, z: i32) -> Self {
		GlobalPosition { x, y, z }
	}

	pub fn local_block(&self) -> CubePosition {
		CubePosition::new(
			((self.x) & 15) as u8,
			  self.y  & 15,
			((self.z) & 15) as u8
		)
	}

	pub fn global_chunk(&self) -> GlobalChunkPosition {
		let (x, y, z) = (
			self.x >> 4,
			self.y >> 4,
			self.z >> 4
		);

		GlobalChunkPosition::new(x, y, z)
	}

	pub fn global_column(&self) -> GlobalColumnPosition {
		let (x, z) = (
			self.x >> 4,
			self.z >> 4
		);

		GlobalColumnPosition::new(x, z)
	}

	pub fn x(&self) -> i32 {
		self.x
	}

	pub fn y(&self) -> u8 {
		self.y
	}

	pub fn z(&self) -> i32 {
		self.z
	}
}

impl Display for GlobalPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
	}
}

// Y << 56 | Z << 28 | X
#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct GlobalChunkPosition(u64);

impl GlobalChunkPosition {
	const X_MASK: u64 = 268435455;
	const NOT_X_MASK: u64 = !Self::X_MASK;

	const Z_MASK: u64 = 268435455 << 28;
	const NOT_Z_MASK: u64 = !Self::Z_MASK;

	const Y_MASK: u64 = 15 << 56;
	const NOT_Y_MASK: u64 = !Self::Y_MASK;

	const  LOWEST_H_VAL: u64 = 134217728;
	const HIGHEST_H_VAL: u64 = 134217727;

	pub fn new(x: i32, y: u8, z: i32) -> Self {
		let (x, y, z) = (
			(x as u64) & MAX_U28,
			(y as u64) & 15,
			(z as u64) & MAX_U28
		);

		GlobalChunkPosition(
			(y << 56) |
			(z << 28) |
			 x
		)
	}

	pub fn from_column(column: GlobalColumnPosition, y: u8) -> Self {
		Self::new(column.x(), y, column.z())
	}

	pub fn x(&self) -> i32 {
		let unsigned = (self.0 & MAX_U28) as i32;
		(unsigned << 4) >> 4
	}

	pub fn y(&self) -> u8 {
		let value = (self.0 >> 56) & 15;

		value as u8
	}

	pub fn z(&self) -> i32 {
		let unsigned = ((self.0 >> 28) & MAX_U28) as i32;
		(unsigned << 4) >> 4
	}

	pub fn column(&self) -> GlobalColumnPosition {
		GlobalColumnPosition(self.0 & MAX_U56)
	}

	pub fn local_chunk(&self) -> CubePosition {
		let (z, x) = (
			(self.0 >> 28) & 15,
			 self.0        & 15
		);

		CubePosition::new(x as u8, self.y(), z as u8)
	}

	pub fn global_sector(&self) -> GlobalSectorPosition {
		GlobalSectorPosition::new(self.x() >> 4, self.z() >> 4)
	}

	pub fn plus_x(&self) -> Option<GlobalChunkPosition> {
		let (keep, x_val) = (self.0 & Self::NOT_X_MASK, self.0 & Self::X_MASK);

		if x_val != Self::HIGHEST_H_VAL {
			let x_val = x_val.wrapping_add(1) & Self::X_MASK;

			Some(GlobalChunkPosition(keep | x_val))
		} else {
			None
		}
	}

	pub fn minus_x(&self) -> Option<GlobalChunkPosition> {
		let (keep, x_val) = (self.0 & Self::NOT_X_MASK, self.0 & Self::X_MASK);

		if x_val != Self::LOWEST_H_VAL {
			let x_val = x_val.wrapping_sub(1) & Self::X_MASK;

			Some(GlobalChunkPosition(keep | x_val))
		} else {
			None
		}
	}

	pub fn plus_z(&self) -> Option<GlobalChunkPosition> {
		let (keep, z_val) = (self.0 & Self::NOT_Z_MASK, self.0 & Self::Z_MASK);

		if z_val != Self::HIGHEST_H_VAL {
			let z_val = z_val.wrapping_add(1 << 28) & Self::Z_MASK;

			Some(GlobalChunkPosition(keep | z_val))
		} else {
			None
		}
	}

	pub fn minus_z(&self) -> Option<GlobalChunkPosition> {
		let (keep, z_val) = (self.0 & Self::NOT_Z_MASK, self.0 & Self::Z_MASK);

		if z_val != Self::LOWEST_H_VAL {
			let z_val = z_val.wrapping_sub(1 << 28) & Self::Z_MASK;

			Some(GlobalChunkPosition(keep | z_val))
		} else {
			None
		}
	}

	pub fn plus_y(&self) -> Option<GlobalChunkPosition> {
		let (keep, y_val) = (self.0 & Self::NOT_Y_MASK, self.0 & Self::Y_MASK);

		if y_val != 15 << 56 {
			let y_val = y_val.wrapping_add(1 << 56) & Self::Y_MASK;

			Some(GlobalChunkPosition(keep | y_val))
		} else {
			None
		}
	}

	pub fn minus_y(&self) -> Option<GlobalChunkPosition> {
		let (keep, y_val) = (self.0 & Self::NOT_Y_MASK, self.0 & Self::Y_MASK);

		if y_val != 0 {
			let y_val = y_val.wrapping_sub(1 << 56) & Self::Y_MASK;

			Some(GlobalChunkPosition(keep | y_val))
		} else {
			None
		}
	}
}

impl Display for GlobalChunkPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {}, {})", self.x(), self.y(), self.z())
	}
}

impl Debug for GlobalChunkPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "GlobalChunkPosition {{ x: {}, y: {}, z: {} }}", self.x(), self.y(), self.z())
	}
}

// Z << 28 | X
#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct GlobalColumnPosition(u64);

impl GlobalColumnPosition {
	pub fn new(x: i32, z: i32) -> Self {
		let (x, z) = (
			(x as u64) & MAX_U28,
			(z as u64) & MAX_U28
		);

		GlobalColumnPosition(
			(z << 28) | x
		)
	}

	pub fn combine(global: GlobalSectorPosition, local: LayerPosition) -> Self {
		Self::new (
			(global.x() << 4) + (local.x() as i32),
			(global.z() << 4) + (local.z() as i32)
		)
	}

	pub fn x(&self) -> i32 {
		let unsigned = (self.0 & MAX_U28) as i32;
		(unsigned << 4) >> 4
	}

	pub fn z(&self) -> i32 {
		let unsigned = ((self.0 >> 28) & MAX_U28) as i32;
		(unsigned << 4) >> 4
	}

	pub fn local_layer(&self) -> LayerPosition {
		let (z, x) = (
			(self.0 >> 28) & 15,
			 self.0        & 15
		);

		LayerPosition::new(x as u8, z as u8)
	}

	pub fn global_sector(&self) -> GlobalSectorPosition {
		GlobalSectorPosition::new(self.x() >> 4, self.z() >> 4)
	}
}

impl Display for GlobalColumnPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {})", self.x(), self.z())
	}
}

impl Debug for GlobalColumnPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "GlobalColumnPosition {{ x: {}, z: {} }}", self.x(), self.z())
	}
}

// This is split up into 3 u16s so that this takes up 48 bits (6 bytes) instead of 64 bits (8 bytes).
#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct GlobalSectorPosition {
	x_high: u16,
	z_high: u16,
	// X << 8 | Z
	zx_low: u16
}

impl GlobalSectorPosition {
	pub fn new(x: i32, z: i32) -> Self {
		let x_low = (x & 255) as u16;
		let z_low = (z & 255) as u16;

		GlobalSectorPosition {
			x_high: (x >> 8) as u16,
			z_high: (z >> 8) as u16,
			zx_low: (z_low << 8) | x_low
		}
	}

	pub fn x(&self) -> i32 {
		let high = (self.x_high as u32) << 8;
		let low  = (self.zx_low as u32) & 255;

		let unsigned = (high | low) as i32;

		(unsigned << 8) >> 8
	}

	pub fn z(&self) -> i32 {
		let high = (self.z_high as u32) << 8;
		let low  = (self.zx_low as u32) >> 8;

		let unsigned = (high | low) as i32;

		(unsigned << 8) >> 8
	}
}

impl Display for GlobalSectorPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "({}, {})", self.x(), self.z())
	}
}

impl Debug for GlobalSectorPosition {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "GlobalSectorPosition {{ x: {}, z: {} }}", self.x(), self.z())
	}
}

#[cfg(test)]
mod test {
	use crate::position::{CubePosition, LayerPosition, GlobalPosition, GlobalChunkPosition, GlobalColumnPosition, GlobalSectorPosition};

	// TODO: Test negative coordinates

	#[test]
	fn test_global_sector() {
		fn try_pair(x: i32, z: i32) {
			assert!(x <= 8388607 && z <= 8388607 && x >= -8388608 && z >= -8388608);

			let position = GlobalSectorPosition::new(x, z);

			assert_eq!(x, position.x(), "X coordinate mismatch");
			assert_eq!(z, position.z(), "Z coordinate mismatch");
		}

		try_pair(1, 1);
		try_pair(832, 3725);

		try_pair(-1, -1);
		try_pair(-832, -3725);
	}

	#[test]
	fn test_global_chunk_relative() {
		// X

		assert_eq!(GlobalChunkPosition::new(0, 0, 0).plus_x(), Some(GlobalChunkPosition::new(1, 0, 0)));
		assert_eq!(GlobalChunkPosition::new(-1, 7, -1).plus_x(), Some(GlobalChunkPosition::new(0, 7, -1)));

		assert_eq!(GlobalChunkPosition::new(0, 0, 0).minus_x(), Some(GlobalChunkPosition::new(-1, 0, 0)));
		assert_eq!(GlobalChunkPosition::new(-1, 7, -1).minus_x(), Some(GlobalChunkPosition::new(-2, 7, -1)));

		// Y

		assert_eq!(GlobalChunkPosition::new(0, 0, 0).plus_y(), Some(GlobalChunkPosition::new(0, 1, 0)));
		assert_eq!(GlobalChunkPosition::new(-1, 15, -1).plus_y(), None);

		assert_eq!(GlobalChunkPosition::new(0, 0, 0).minus_y(), None);
		assert_eq!(GlobalChunkPosition::new(-1, 7, -1).minus_y(), Some(GlobalChunkPosition::new(-1, 6, -1)));

		// Z

		assert_eq!(GlobalChunkPosition::new(0, 0, 0).plus_z(), Some(GlobalChunkPosition::new(0, 0, 1)));
		assert_eq!(GlobalChunkPosition::new(-1, 7, -1).plus_z(), Some(GlobalChunkPosition::new(-1, 7, 0)));

		assert_eq!(GlobalChunkPosition::new(0, 0, 0).minus_z(), Some(GlobalChunkPosition::new(0, 0, -1)));
		assert_eq!(GlobalChunkPosition::new(-1, 7, -1).minus_z(), Some(GlobalChunkPosition::new(-1, 7, -2)));
	}

	#[test]
	fn test_conversion() {
		let block = GlobalPosition::new(213179, 44, 952109);

		let (local_block, global_chunk, global_column) = (block.local_block(), block.global_chunk(), block.global_column());

		assert_eq!(local_block, CubePosition::new(11, 12, 13));
		assert_eq!(global_chunk, GlobalChunkPosition::new(13323, 2, 59506));
		assert_eq!(global_column, GlobalColumnPosition::new(13323, 59506));

		let (local_chunk, local_column, global_sector) = (global_chunk.local_chunk(), global_column.local_layer(), global_chunk.global_sector());

		assert_eq!(local_chunk, CubePosition::new(11, 2, 2));
		assert_eq!(local_column, LayerPosition::new(11, 2));
		assert_eq!(global_sector, GlobalSectorPosition::new(832, 3719));
	}
}