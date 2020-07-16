use std::mem;
use vocs::component::{ChunkStorage, LayerStorage};
use vocs::mask::{ChunkMask, LayerMask, Mask};
use vocs::position::{dir, ChunkPosition};
use vocs::view::{Directional, MaskOffset, SpillChunkMask, SplitDirectional};

/// A double-buffered queue. Useful for breadth-first search algorithms.
#[derive(Clone)]
pub struct ChunkQueue {
	front: ChunkMask,
	back: SpillChunkMask,
}

impl ChunkQueue {
	pub fn new() -> ChunkQueue {
		ChunkQueue { front: ChunkMask::default(), back: SpillChunkMask::default() }
	}

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

	pub fn reset_spills(&mut self) -> ChunkSpills {
		let mut spills = ChunkSpills::default();

		mem::swap(&mut spills.0, &mut self.back.spills);

		spills
	}

	pub fn pop_first(&mut self) -> Option<ChunkPosition> {
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

pub struct ChunkSpills(Directional<LayerMask>);

impl ChunkSpills {
	pub(crate) fn split(self) -> SplitDirectional<LayerMask> {
		self.0.split()
	}
}

impl Default for ChunkSpills {
	fn default() -> ChunkSpills {
		ChunkSpills(Directional::combine(SplitDirectional {
			plus_x: LayerMask::default(),
			minus_x: LayerMask::default(),
			plus_z: LayerMask::default(),
			minus_z: LayerMask::default(),
			up: LayerMask::default(),
			down: LayerMask::default(),
		}))
	}
}
