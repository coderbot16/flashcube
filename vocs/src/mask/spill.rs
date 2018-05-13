use position::ChunkPosition;
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
		match position.plus_y() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.up.set_true(position.layer())
		}
	}

	pub fn set_up_false(&mut self, position: ChunkPosition) {
		match position.plus_y() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.up.set_false(position.layer())
		}
	}

	pub fn set_down_true(&mut self, position: ChunkPosition) {
		match position.minus_y() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.down.set_true(position.layer())
		}
	}

	pub fn set_down_false(&mut self, position: ChunkPosition) {
		match position.minus_y() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.down.set_false(position.layer())
		}
	}

	// X

	pub fn set_plus_x_true(&mut self, position: ChunkPosition) {
		match position.plus_x() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.plus_x.set_true(position.layer_yz())
		}
	}

	pub fn set_plus_x_false(&mut self, position: ChunkPosition) {
		match position.plus_x() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.plus_x.set_false(position.layer_yz())
		}
	}

	pub fn set_minus_x_true(&mut self, position: ChunkPosition) {
		match position.minus_x() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.minus_x.set_true(position.layer_yz())
		}
	}

	pub fn set_minus_x_false(&mut self, position: ChunkPosition) {
		match position.minus_x() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.minus_x.set_false(position.layer_yz())
		}
	}

	// Z

	pub fn set_plus_z_true(&mut self, position: ChunkPosition) {
		match position.plus_z() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.plus_z.set_true(position.layer_yx())
		}
	}

	pub fn set_plus_z_false(&mut self, position: ChunkPosition) {
		match position.plus_z() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.plus_z.set_false(position.layer_yx())
		}
	}

	pub fn set_minus_z_true(&mut self, position: ChunkPosition) {
		match position.minus_z() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.minus_z.set_true(position.layer_yx())
		}
	}

	pub fn set_minus_z_false(&mut self, position: ChunkPosition) {
		match position.minus_z() {
			Some(position) => self.mask.set_true(position),
			None => self.spills.minus_z.set_false(position.layer_yx())
		}
	}
}