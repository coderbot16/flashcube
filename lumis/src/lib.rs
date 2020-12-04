pub mod heightmap;
pub mod light;
mod monolith;
pub mod queue;
pub mod sources;

pub use heightmap::compute_world_heightmaps;
pub use monolith::{compute_world_skylight, compute_world_blocklight, IgnoreTraces, PrintTraces, LightTraces};

use vocs::component::CubeStorage;
use vocs::nibbles::{u4, NibbleCube};
use vocs::position::CubePosition;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PackedNibbleCube {
	Unpacked(NibbleCube),
	EntirelyDark,
	EntirelyLit
}

impl PackedNibbleCube {
	pub fn get(&self, position: CubePosition) -> u4 {
		match self {
			&PackedNibbleCube::Unpacked(ref unpacked) => unpacked.get(position),
			&PackedNibbleCube::EntirelyDark => u4::ZERO,
			&PackedNibbleCube::EntirelyLit => u4::MAX
		}
	}

	pub fn is_packed(&self) -> bool {
		match self {
			&PackedNibbleCube::Unpacked(_) => false,
			&PackedNibbleCube::EntirelyDark => true,
			&PackedNibbleCube::EntirelyLit => true
		}
	}

	pub fn set(&mut self, position: CubePosition, value: u4) {
		match self {
			&mut PackedNibbleCube::Unpacked(ref mut unpacked) => {
				unpacked.set(position, value);

				return
			},
			&mut PackedNibbleCube::EntirelyDark => if value == u4::ZERO {
				return
			},
			&mut PackedNibbleCube::EntirelyLit => if value == u4::MAX {
				return
			}
		}

		// Slow path: Need to allocate the backing array
		let mut data = NibbleCube::default();
		data.set(position, value);

		*self = PackedNibbleCube::Unpacked(data);
	}

	pub fn unpack(self) -> NibbleCube {
		match self {
			PackedNibbleCube::Unpacked(data) => data,
			PackedNibbleCube::EntirelyDark => NibbleCube::default(),
			PackedNibbleCube::EntirelyLit => {
				let mut data = NibbleCube::default();

				data.fill(u4::MAX);

				data
			}
		}
	}

	pub fn unpack_in_place(&mut self) {
		let filled = match self {
			PackedNibbleCube::Unpacked(_) => return,
			PackedNibbleCube::EntirelyDark => NibbleCube::default(),
			PackedNibbleCube::EntirelyLit => {
				let mut data = NibbleCube::default();

				data.fill(u4::MAX);

				data
			}
		};

		*self = PackedNibbleCube::Unpacked(filled);
	}
}

impl Default for PackedNibbleCube {
	fn default() -> Self {
		PackedNibbleCube::EntirelyDark
	}
}
