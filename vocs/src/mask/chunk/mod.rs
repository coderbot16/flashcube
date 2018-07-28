use mask::{Mask, u1x64};
use component::ChunkStorage;
use position::{ChunkPosition, dir, Offset};
use std::ops::Index;
use std::cmp::PartialEq;

mod scan;
mod layer_view;

pub use self::scan::*;
pub use self::layer_view::*;

// Hackish constants for implementing Index on bit packed structures.
const FALSE_REF: &bool = &false;
const TRUE_REF:  &bool = &true;

pub struct ChunkMask {
	blocks: Box<[u1x64; 64]>,
	inhabited: u1x64
}

impl ChunkMask {
	pub fn combine(&mut self, other: &ChunkMask) {
		for (target, other) in self.blocks.iter_mut().zip(other.blocks.iter()) {
			*target = *target | *other;
		}

		self.inhabited |= other.inhabited;
	}

	#[inline]
	pub fn set_neighbors(&mut self, position: ChunkPosition) {
		self.set_h_neighbors(position);
		position.offset(dir::Down  ).map(|at| self.set_true(at));
		position.offset(dir::Up    ).map(|at| self.set_true(at));
	}

	#[inline]
	pub fn set_h_neighbors(&mut self, position: ChunkPosition) {
		position.offset(dir::MinusX).map(|at| self.set_true(at));
		position.offset(dir::PlusX ).map(|at| self.set_true(at));
		position.offset(dir::MinusZ).map(|at| self.set_true(at));
		position.offset(dir::PlusZ ).map(|at| self.set_true(at));
	}

	pub fn pop_first(&mut self) -> Option<ChunkPosition> {
		let block_index = self.inhabited.first_bit();

		if block_index > 63 {
			return None;
		}

		let first = &mut self.blocks[block_index as usize];

		let sub_index = first.first_bit();
		*first = first.clear(sub_index);

		self.inhabited = self.inhabited.replace(block_index, !first.empty());

		Some(ChunkPosition::from_yzx(
			((block_index as u16) * 64) |
				  (  sub_index as u16)
		))
	}

	pub fn blocks(&self) -> &[u1x64; 64] {
		&self.blocks
	}

	pub fn empty(&self) -> bool {
		self.inhabited.empty()
	}

	pub fn layer_zx_mut(&mut self, y: u8) -> LayerZxMut {
		let y = y & 15;
		let start = (y as usize) * 4;

		LayerZxMut::from_slice(&mut self.blocks[start..start+4], &mut self.inhabited, y * 4)
	}

	pub fn layer_zy_mut(&mut self, x: u8) -> LayerZyMut {
		LayerZyMut::from_mask(self, x)
	}

	pub fn layer_yx_mut(&mut self, z: u8) -> LayerYxMut {
		LayerYxMut::from_mask(self, z)
	}
}

impl ChunkStorage<bool> for ChunkMask {
	fn get(&self, position: ChunkPosition) -> bool {
		self[position]
	}

	fn set(&mut self, position: ChunkPosition, value: bool) {
		let index = position.yzx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		let block = self.blocks[block_index].replace(sub_index as u8, value);

		self.blocks[block_index] = block;
		self.inhabited = self.inhabited.replace(block_index as u8, !block.empty());
	}

	fn fill(&mut self, value: bool) {
		let fill = u1x64::splat(value);

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

		self.blocks[block_index] = self.blocks[block_index].set(sub_index as u8);
		self.inhabited = self.inhabited.set(block_index as u8);
	}

	fn set_false(&mut self, position: ChunkPosition) {
		let index = position.yzx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		let cleared = self.blocks[block_index].clear(sub_index as u8);

		self.blocks[block_index] = cleared;
		self.inhabited = self.inhabited.replace(block_index as u8, !cleared.empty());

	}

	fn set_or(&mut self, position: ChunkPosition, value: bool) {
		let index = position.yzx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		self.blocks[block_index] = self.blocks[block_index].replace_or(sub_index as u8, value);
		self.inhabited = self.inhabited.replace_or(block_index as u8, value);
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
		let (block_index, sub_index) = (index / 64, index % 64);

		if self.blocks[block_index].extract(sub_index as u8) { TRUE_REF } else { FALSE_REF }
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

impl PartialEq for ChunkMask {
	fn eq(&self, other: &Self) -> bool {
		if self.inhabited != other.inhabited {
			 return false;
		}

		(&self.blocks[..]) == (&other.blocks[..])
	}
}

impl Eq for ChunkMask {}

impl Default for ChunkMask {
	fn default() -> Self {
		ChunkMask {
			blocks: Box::new([u1x64::splat(false); 64]),
			inhabited: u1x64::splat(false)
		}
	}
}