use vocs::mask::{Mask, ChunkMask, LayerMask};
use vocs::position::{ChunkPosition, dir};
use vocs::component::*;
use vocs::view::{SpillChunkMask, MaskOffset, SplitDirectional, Directional};
use std::mem;

/// A double-buffered queue. Useful for breadth-first search algorithms.
#[derive(Clone, Default)]
pub struct Queue {
	front:      ChunkMask,
	back:  SpillChunkMask
}

impl Queue {
	pub fn clear(&mut self) {
		self.front.fill(false);
		self.back.primary.fill(false);
		self.back.spills[dir::Up].fill(false);
		self.back.spills[dir::Down].fill(false);
		self.back.spills[dir::PlusX].fill(false);
		self.back.spills[dir::MinusX].fill(false);
		self.back.spills[dir::PlusZ].fill(false);
		self.back.spills[dir::MinusZ].fill(false);
	}

	pub fn reset_from_mask(&mut self, front: ChunkMask) {
		self.front.fill(false);

		self.back.primary = front;
		self.back.spills[dir::Up].fill(false);
		self.back.spills[dir::Down].fill(false);
		self.back.spills[dir::PlusX].fill(false);
		self.back.spills[dir::MinusX].fill(false);
		self.back.spills[dir::PlusZ].fill(false);
		self.back.spills[dir::MinusZ].fill(false);
	}

	pub fn reset_spills(&mut self) -> Directional<LayerMask> {
		let mut spills = Directional::combine(SplitDirectional {
			plus_x: LayerMask::default(),
			minus_x: LayerMask::default(),
			plus_z: LayerMask::default(),
			minus_z: LayerMask::default(),
			up: LayerMask::default(),
			down: LayerMask::default()
		});

		mem::swap(&mut spills, &mut self.back.spills);

		spills
	}

	pub fn next(&mut self) -> Option<ChunkPosition> {
		self.front.pop_first()
	}

	pub fn flip(&mut self) -> bool {
		mem::swap(&mut self.front, &mut self.back.primary);

		!self.front.empty()
	}

	pub fn enqueue(&mut self, position: ChunkPosition) {
		self.back.primary.set_true(position)
	}

	pub fn enqueue_neighbors(&mut self, position: ChunkPosition) {
		self.enqueue_h_neighbors(position);
		self.back.set_offset_true(position, dir::Down);
		self.back.set_offset_true(position, dir::Up);
	}

	pub fn enqueue_h_neighbors(&mut self, position: ChunkPosition) {
		self.back.set_offset_true(position, dir::MinusX);
		self.back.set_offset_true(position, dir::MinusZ);
		self.back.set_offset_true(position, dir::PlusX);
		self.back.set_offset_true(position, dir::PlusZ);
	}

	pub fn mask_mut(&mut self) -> &mut SpillChunkMask {
		&mut self.back
	}
}
