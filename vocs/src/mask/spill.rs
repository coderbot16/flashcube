use position::{ChunkPosition, Offset, Up, Down, PlusX, MinusX, PlusZ, MinusZ};
use mask::{Mask, LayerMask, ChunkMask};
use view::SplitDirectional;

pub type Spills = SplitDirectional<LayerMask>;

#[derive(Default, Clone)]
pub struct SpillChunkMask {
	pub mask: ChunkMask,
	pub spills: Spills
}

impl SpillChunkMask {
	// Y

	pub fn set_up_true(&mut self, position: ChunkPosition) {
		match position.offset(Up) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.up.set_true(position.layer())
		}
	}

	pub fn set_up_false(&mut self, position: ChunkPosition) {
		match position.offset(Up) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.up.set_false(position.layer())
		}
	}

	pub fn set_down_true(&mut self, position: ChunkPosition) {
		match position.offset(Down) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.down.set_true(position.layer())
		}
	}

	pub fn set_down_false(&mut self, position: ChunkPosition) {
		match position.offset(Down) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.down.set_false(position.layer())
		}
	}

	// X

	pub fn set_plus_x_true(&mut self, position: ChunkPosition) {
		match position.offset(PlusX) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.plus_x.set_true(position.layer_yz())
		}
	}

	pub fn set_plus_x_false(&mut self, position: ChunkPosition) {
		match position.offset(PlusX) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.plus_x.set_false(position.layer_yz())
		}
	}

	pub fn set_minus_x_true(&mut self, position: ChunkPosition) {
		match position.offset(MinusX) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.minus_x.set_true(position.layer_yz())
		}
	}

	pub fn set_minus_x_false(&mut self, position: ChunkPosition) {
		match position.offset(MinusX) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.minus_x.set_false(position.layer_yz())
		}
	}

	// Z

	pub fn set_plus_z_true(&mut self, position: ChunkPosition) {
		match position.offset(PlusZ) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.plus_z.set_true(position.layer_yx())
		}
	}

	pub fn set_plus_z_false(&mut self, position: ChunkPosition) {
		match position.offset(PlusZ) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.plus_z.set_false(position.layer_yx())
		}
	}

	pub fn set_minus_z_true(&mut self, position: ChunkPosition) {
		match position.offset(MinusZ) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.minus_z.set_true(position.layer_yx())
		}
	}

	pub fn set_minus_z_false(&mut self, position: ChunkPosition) {
		match position.offset(MinusZ) {
			Some(position) => self.mask.set_true(position),
			None => self.spills.minus_z.set_false(position.layer_yx())
		}
	}
}