use mask::Mask;
use component::ChunkStorage;
use position::{ChunkPosition, dir, Offset};
use std::ops::Index;

mod scan;

pub use self::scan::*;

// Hackish constants for implementing Index on bit packed structures.
const FALSE_REF: &bool = &false;
const TRUE_REF:  &bool = &true;

pub struct ChunkMask {
	blocks: Box<[u64; 64]>,
	inhabited: u64
}

impl ChunkMask {
	pub fn combine(&mut self, other: &ChunkMask) {
		for (target, other) in self.blocks.iter_mut().zip(other.blocks.iter()) {
			*target = *target | *other;
		}

		self.inhabited |= other.inhabited;
	}

	pub fn set_neighbors(&mut self, position: ChunkPosition) {
		position.offset(dir::MinusX).map(|at| self.set_true(at));
		position.offset(dir::PlusX ).map(|at| self.set_true(at));
		position.offset(dir::MinusZ).map(|at| self.set_true(at));
		position.offset(dir::PlusZ ).map(|at| self.set_true(at));
		position.offset(dir::Down  ).map(|at| self.set_true(at));
		position.offset(dir::Up    ).map(|at| self.set_true(at));
	}

	pub fn blocks(&self) -> &[u64; 64] {
		&self.blocks
	}
}

impl ChunkStorage<bool> for ChunkMask {
	fn get(&self, position: ChunkPosition) -> bool {
		self[position]
	}

	fn set(&mut self, position: ChunkPosition, value: bool) {
		<Self as Mask<ChunkPosition>>::set(self, position, value);
	}

	fn fill(&mut self, value: bool) {
		let fill = if value { u64::max_value() } else { 0 };

		for value in self.blocks.iter_mut() {
			*value = fill;
		}

		self.inhabited = fill;
	}
}

impl Mask<ChunkPosition> for ChunkMask {
	fn set_true(&mut self, position: ChunkPosition) {
		let index = position.yzx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		self.inhabited |= 1 << block_index;
		self.blocks[block_index] |= 1 << sub_index;
	}

	fn set_false(&mut self, position: ChunkPosition) {
		let index = position.yzx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		let cleared = self.blocks[block_index] & !(1 << sub_index);
		self.blocks[block_index] = cleared;

		let cleared_inhabited = self.inhabited & !(1 << block_index);
		self.inhabited = cleared_inhabited | (((cleared != 0) as u64) << block_index);
	}

	fn set_or(&mut self, position: ChunkPosition, value: bool) {
		let index = position.yzx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		self.inhabited |= (value as u64) << block_index;
		self.blocks[block_index] |= (value as u64) << sub_index;
	}

	fn set(&mut self, position: ChunkPosition, value: bool) {
		let index = position.yzx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		let cleared = self.blocks[block_index] & !(1 << sub_index);
		let block = cleared | ((value as u64) << sub_index);

		// Update inhabited, using a similar method to updating the bitfield.
		let cleared_inhabited = self.inhabited & !(1 << block_index);
		self.inhabited = cleared_inhabited | (((block != 0) as u64) << block_index);

		self.blocks[block_index] = block;
	}

	fn count_ones(&self) -> u32 {
		self.blocks.iter().fold(0, |state, value| state + value.count_ones())
	}

	fn count_zeros(&self) -> u32 {
		self.blocks.iter().fold(0, |state, value| state + value.count_zeros())
	}
}

impl Index<ChunkPosition> for ChunkMask {
	type Output = bool;

	fn index(&self, position: ChunkPosition) -> &bool {
		let index = position.yzx() as usize;

		if (self.blocks[index / 64] >> (index % 64))&1 == 1 { TRUE_REF } else { FALSE_REF }
	}
}

impl Clone for ChunkMask {
	fn clone(&self) -> Self {
		ChunkMask {
			blocks: Box::new([
				self.blocks[ 0], self.blocks[ 1], self.blocks[ 2], self.blocks[ 3], self.blocks[ 4], self.blocks[ 5], self.blocks[ 6], self.blocks[ 7], self.blocks[ 8], self.blocks[ 9],
				self.blocks[10], self.blocks[11], self.blocks[12], self.blocks[13], self.blocks[14], self.blocks[15], self.blocks[16], self.blocks[17], self.blocks[18], self.blocks[19],
				self.blocks[20], self.blocks[21], self.blocks[22], self.blocks[23], self.blocks[24], self.blocks[25], self.blocks[26], self.blocks[27], self.blocks[28], self.blocks[29],
				self.blocks[30], self.blocks[31], self.blocks[32], self.blocks[33], self.blocks[34], self.blocks[35], self.blocks[36], self.blocks[37], self.blocks[38], self.blocks[39],
				self.blocks[40], self.blocks[41], self.blocks[42], self.blocks[43], self.blocks[44], self.blocks[45], self.blocks[46], self.blocks[47], self.blocks[48], self.blocks[49],
				self.blocks[50], self.blocks[51], self.blocks[52], self.blocks[53], self.blocks[54], self.blocks[55], self.blocks[56], self.blocks[57], self.blocks[58], self.blocks[59],
				self.blocks[60], self.blocks[61], self.blocks[62], self.blocks[63]
			]),
			inhabited: self.inhabited
		}
	}
}

impl Default for ChunkMask {
	fn default() -> Self {
		ChunkMask {
			blocks: Box::new([0; 64]),
			inhabited: 0
		}
	}
}