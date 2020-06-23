use std::convert::TryFrom;
use vocs::component::{ChunkStorage, LayerStorage};
use vocs::mask::{ChunkMask, LayerMask, Mask};
use vocs::position::ChunkPosition;

/// Effectively a write once, read once Sector tailored for storing LayerMasks.
pub struct DirectionSpills {
	indices: Box<[u16; 4096]>,
	masks: Vec<LayerMask>,
	filled: ChunkMask
}

impl DirectionSpills {
	pub fn new() -> Self {
		DirectionSpills {
			indices: Box::new([0u16; 4096]),
			masks: Vec::with_capacity(128),
			filled: ChunkMask::default()
		}
	}

	pub fn set(&mut self, position: ChunkPosition, spill: LayerMask) {
		if spill.is_filled(false) {
			return;
		}

		if spill.is_filled(true) {
			self.filled.set_true(position);
			return;
		}

		let index = u16::try_from(self.masks.len()).expect("Inserted too many directional spills without clearing!");

		if index == u16::max_value() {
			panic!("Inserted too many directional spills without clearing!");
		}

		// Add 1 because 0 indicates a "null" entry
		self.indices[position.yzx() as usize] = index + 1;

		self.masks.push(spill);
	}

	pub fn get(&self, position: ChunkPosition) -> Option<LayerMask> {
		let index = self.indices[position.yzx() as usize];

		if index == 0 {
			let mut mask = LayerMask::default();

			if self.filled[position] {
				mask.fill(true);

				return Some(mask);
			} else {
				return None;
			}
		}

		Some(self.masks[index as usize - 1].clone())
	}

	pub fn reset(&mut self) {
		*self.indices = [0u16; 4096];
		self.masks.clear();
		self.filled.fill(false);
	}
}
