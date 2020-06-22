use std::convert::TryFrom;
use vocs::component::{ChunkStorage, LayerStorage};
use vocs::mask::{ChunkMask, LayerMask, Mask};
use vocs::position::{ChunkPosition, dir, Offset};

struct DirectionSpills {
	indices: Box<[u16; 4096]>,
	masks: Vec<LayerMask>,
	defaults: ChunkMask
}

impl DirectionSpills {
	fn set(&mut self, position: ChunkPosition, spill: LayerMask) {
		if spill.is_filled(false) {
			return;
		}

		if spill.is_filled(true) {
			self.defaults.set_true(position);
			return;
		}

		let index = u16::try_from(self.masks.len()).expect("Inserted too many directional spills without clearing!");

		self.masks.push(spill);

		if index == u16::max_value() {
			panic!("Inserted too many directional spills without clearing!");
		}

		// Add 1 because 0 indicates no mask in the masks array
		self.indices[position.yzx() as usize] = index + 1;
	}

	fn get(&mut self, position: ChunkPosition) -> LayerMask {
		let index = self.indices[position.yzx() as usize];

		if index == 0 {
			let mut mask = LayerMask::default();

			if self.defaults[position] {
				mask.fill(true);
			}

			return mask;
		}

		self.masks[index as usize - 1].clone()
	}

	fn reset(&mut self) {
		*self.indices = [0u16; 4096];
		self.masks.clear();
		self.defaults.fill(false);
	}
}

struct DirectionSpillsSetter<D> where ChunkPosition: Offset<D> {
	direction: D,
	local: DirectionSpills,
	neighbor: DirectionSpills
}
