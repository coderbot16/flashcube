use crate::mask::{u1x64, ChunkMask, LayerMask};
use crate::position::{ChunkPosition, LayerPosition};
use crate::component::*;

pub struct LayerZxMut<'l> {
	layer: &'l mut [u1x64],
	inhabited: &'l mut u1x64,
	inhabited_offset: u8
}

impl<'l> LayerZxMut<'l> {
	pub fn from_slice(layer: &'l mut [u1x64], inhabited: &'l mut u1x64, inhabited_offset: u8) -> Self {
		assert_eq!(layer.len(), 4);

		LayerZxMut { layer, inhabited, inhabited_offset }
	}

	pub fn combine(&mut self, other: &LayerMask) {
		assert_eq!(self.layer.len(), 4);

		self.layer[0] |= u1x64::from_bits(other.blocks()[0]);
		self.layer[1] |= u1x64::from_bits(other.blocks()[1]);
		self.layer[2] |= u1x64::from_bits(other.blocks()[2]);
		self.layer[3] |= u1x64::from_bits(other.blocks()[3]);

		*self.inhabited = self.inhabited
			.replace(self.inhabited_offset,     !self.layer[0].empty())
			.replace(self.inhabited_offset + 1, !self.layer[1].empty())
			.replace(self.inhabited_offset + 2, !self.layer[2].empty())
			.replace(self.inhabited_offset + 3, !self.layer[3].empty());
	}
}

impl<'l> LayerStorage<bool> for LayerZxMut<'l> {
	fn get(&self, position: LayerPosition) -> bool {
		let index = position.zx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		self.layer[block_index].extract(sub_index as u8)
	}

	fn is_filled(&self, value: bool) -> bool {
		let term = u1x64::splat(value);

		self.layer == &[term, term, term, term]
	}

	fn set(&mut self, position: LayerPosition, value: bool) {
		let index = position.zx() as usize;
		let (block_index, sub_index) = (index / 64, index % 64);

		let block = self.layer[block_index].replace(sub_index as u8, value);

		self.layer[block_index] = block;
		*self.inhabited = self.inhabited.replace(self.inhabited_offset + block_index as u8, !block.empty());
	}

	fn fill(&mut self, value: bool) {
		let inhabited_mask = u1x64::from_bits(0xF << self.inhabited_offset);

		if value {
			*self.inhabited |= inhabited_mask;
		} else {
			*self.inhabited &= !inhabited_mask;
		}

		let value = u1x64::splat(value);

		assert_eq!(self.layer.len(), 4);

		self.layer[0] = value;
		self.layer[1] = value;
		self.layer[2] = value;
		self.layer[3] = value;
	}
}

pub struct LayerZyMut<'l> {
	mask: &'l mut ChunkMask,
	x: u8
}

impl<'l> LayerZyMut<'l> {
	pub fn from_mask(mask: &'l mut ChunkMask, x: u8) -> Self {
		LayerZyMut { mask, x }
	}

	pub fn combine(&mut self, other: &LayerMask) {
		// Clear inhabited, we'll regenerate it in the loop.
		let mut inhabited = u1x64::default();

		for (index, block) in self.mask.blocks.iter_mut().enumerate() {
			let z_offset = ((index % 4) * 4) as u8;
			let y = (index / 4) as u8;

			let combined = block
				 .replace_or(self.x,      other.get(LayerPosition::new(y, z_offset    )))
			     .replace_or(self.x + 16, other.get(LayerPosition::new(y, z_offset + 1)))
			     .replace_or(self.x + 32, other.get(LayerPosition::new(y, z_offset + 2)))
			     .replace_or(self.x + 48, other.get(LayerPosition::new(y, z_offset + 3)));

			*block = combined;

			inhabited = inhabited.replace(index as u8, !combined.empty());
		}

		self.mask.inhabited = inhabited;
	}

	fn bitmask(&self) -> u1x64 {
		u1x64::from_bits((1 | (1 << 16) | (1 << 32) | (1 << 48)) << self.x)
	}
}

impl<'l> LayerStorage<bool> for LayerZyMut<'l> {
	fn get(&self, position: LayerPosition) -> bool {
		self.mask.get(ChunkPosition::new(self.x, position.x(), position.z()))
	}

	fn is_filled(&self, value: bool) -> bool {
		let bitmask = self.bitmask();

		if value {
			let mut collected = bitmask;

			// This loop exploits the properties of bitwise AND to accomplish a comparison
			// of a certain set of bits without a branch. It selects only the bits involved in the layer,
			// and then uses bitwise AND between that and the collected value. An unset bit in the selected
			// bits, indicating that it is not filled with true bits, would cause the corresponding bit in
			// collected to become false. This bit cannot turn true again, and this is detected by the
			// final comparison.
			//
			// This is a similar technique as the one used in crypto to implement branchless memcmp.

			for &block in self.mask.blocks().iter() {
				collected &= block & bitmask;
			}

			collected == bitmask
		} else {
			let mut collected = u1x64::default();

			// Similarly to the previous loop, this exploits the properties of a bitwise
			// operation. But, it uses bitwise OR instead, and instead of bits starting off as set,
			// they start off as unset. Singular set bits are detected with the comparison at the end,
			// in a similar manner.

			for &block in self.mask.blocks().iter() {
				collected |= block & bitmask;
			}

			collected == u1x64::default()
		}
	}

	fn set(&mut self, position: LayerPosition, value: bool) {
		self.mask.set(ChunkPosition::new(self.x, position.x(), position.z()), value)
	}

	fn fill(&mut self, value: bool) {
		let bitmask = self.bitmask();
		let bitmask_inverted = !bitmask;

		self.mask.inhabited = u1x64::splat(value);

		if value {
			for block in self.mask.blocks.iter_mut() {
				*block |= bitmask;
			}
		} else {
			for (index, block) in self.mask.blocks.iter_mut().enumerate() {
				*block &= bitmask_inverted;

				self.mask.inhabited = self.mask.inhabited.replace_or(index as u8, *block != u1x64::splat(false))
			}
		}
	}
}

pub struct LayerYxMut<'l> {
	mask: &'l mut ChunkMask,
	z: u8
}

impl<'l> LayerYxMut<'l> {
	pub fn from_mask(mask: &'l mut ChunkMask, z: u8) -> Self {
		LayerYxMut { mask, z }
	}

	pub fn combine(&mut self, other: &LayerMask) {
		let block_offset = self.z / 4;
		let shift = (self.z % 4) * 16;

		// Clear the relevant bits. They will be fixed by the loop.
		self.mask.inhabited &= !self.bitmask();

		for y in 0..16 {
			// Fetch the relevant 16 bits from the source mask.
			let mut source = other.blocks()[(y as usize)/4];
			source >>= 16 * (y % 4);
			source &= 0xFFFF;

			let index = y*4 + block_offset;
			let block = &mut self.mask.blocks[index as usize];

			*block |= u1x64::from_bits(source << shift);

			self.mask.inhabited = self.mask.inhabited.replace_or(index, !block.empty());
		}
	}

	fn bitmask(&self) -> u1x64 {
		u1x64::from_bits(0x1111_1111_1111_1111u64 << (self.z / 4))
	}
}

impl<'l> LayerStorage<bool> for LayerYxMut<'l> {
	fn get(&self, position: LayerPosition) -> bool {
		self.mask.get(ChunkPosition::new(position.x(), position.z(), self.z))
	}

	fn is_filled(&self, value: bool) -> bool {
		let block_offset = self.z / 4;
		let bit_pattern = u1x64::from_bits(0xFFFF << ((self.z % 4) * 16));

		if value {
			let mut collected = bit_pattern;

			for y in 0..16 {
				let index = y*4 + block_offset;
				collected &= self.mask.blocks[index as usize] & bit_pattern;
			}

			collected == bit_pattern
		} else {
			let mut collected = u1x64::default();

			for y in 0..16 {
				let index = y*4 + block_offset;
				collected |= self.mask.blocks[index as usize] & bit_pattern;
			}

			collected == u1x64::default()
		}
	}

	fn set(&mut self, position: LayerPosition, value: bool) {
		self.mask.set(ChunkPosition::new(position.x(), position.z(), self.z), value)
	}

	fn fill(&mut self, value: bool) {
		let block_offset = self.z / 4;
		let bit_pattern = u1x64::from_bits(0xFFFF << ((self.z % 4) * 16));

		if value {
			// Set the relevant bits outside the loop.
			self.mask.inhabited |= self.bitmask();

			for y in 0..16 {
				let index = y*4 + block_offset;
				self.mask.blocks[index as usize] |= bit_pattern;
			}
		} else {
			// Clear the relevant bits. They will be fixed by the loop.
			self.mask.inhabited &= !self.bitmask();

			for y in 0..16 {
				let index = y*4 + block_offset;
				let block = &mut self.mask.blocks[index as usize];

				*block &= !bit_pattern;
				self.mask.inhabited = self.mask.inhabited.replace_or(index, !block.empty());
			}
		}
	}
}

#[cfg(test)]
mod test {
	use crate::mask::{LayerMask, ChunkMask};
	use crate::position::ChunkPosition;
	use crate::component::*;

	fn verify_masks_equal(direct: &ChunkMask, indirect: &ChunkMask) {
		if direct != indirect {
			println!("Error in layer view implementation, indirect method should be identical to direct method!");

			println!("direct inhabited: {:016X}, indirect inhabited: {:016X}", direct.inhabited.to_bits(), indirect.inhabited.to_bits());

			println!("blocks listing: direct, indirect:");
			for (dblock, iblock) in direct.blocks().iter().zip(indirect.blocks.iter()) {
				println!("{:016X}, {:016X}", dblock.to_bits(), iblock.to_bits());
			}

			panic!("Indirect method should be identical to direct method");
		}
	}

	#[test]
	fn test_zx_view() {
		let mut direct = ChunkMask::default();

		for z in 0..16 {
			for x in 0..16 {
				direct.set(ChunkPosition::new(x, 13, z), true);
			}
		}

		let mut indirect = ChunkMask::default();
		indirect.layer_zx_mut(13).fill(true);

		assert!(indirect.layer_zx_mut(13).is_filled(true));
		verify_masks_equal(&direct, &indirect);

		indirect.layer_zx_mut(13).fill(false);

		verify_masks_equal(&ChunkMask::default(), &indirect);

		indirect.layer_zx_mut(13).combine(&LayerMask::default());
		verify_masks_equal(&ChunkMask::default(), &indirect);

		let mut layer_filled = LayerMask::default();
		layer_filled.fill(true);
		indirect.layer_zx_mut(13).combine(&layer_filled);

		verify_masks_equal(&direct, &indirect);
	}

	#[test]
	fn test_zy_view() {
		let mut direct = ChunkMask::default();

		for z in 0..16 {
			for y in 0..16 {
				direct.set(ChunkPosition::new(13, y, z), true);
			}
		}

		let mut indirect = ChunkMask::default();
		indirect.layer_zy_mut(13).fill(true);

		assert!(indirect.layer_zy_mut(13).is_filled(true));
		verify_masks_equal(&direct, &indirect);

		indirect.layer_zy_mut(13).fill(false);

		verify_masks_equal(&ChunkMask::default(), &indirect);

		indirect.layer_zy_mut(13).combine(&LayerMask::default());
		verify_masks_equal(&ChunkMask::default(), &indirect);

		let mut layer_filled = LayerMask::default();
		layer_filled.fill(true);
		indirect.layer_zy_mut(13).combine(&layer_filled);

		verify_masks_equal(&direct, &indirect);
	}

	#[test]
	fn test_yx_view() {
		let mut direct = ChunkMask::default();

		for y in 0..16 {
			for x in 0..16 {
				direct.set(ChunkPosition::new(x, y, 13), true);
			}
		}

		let mut indirect = ChunkMask::default();
		indirect.layer_yx_mut(13).fill(true);

		assert!(indirect.layer_yx_mut(13).is_filled(true));
		verify_masks_equal(&direct, &indirect);

		indirect.layer_yx_mut(13).fill(false);

		verify_masks_equal(&ChunkMask::default(), &indirect);

		indirect.layer_yx_mut(13).combine(&LayerMask::default());
		verify_masks_equal(&ChunkMask::default(), &indirect);

		let mut layer_filled = LayerMask::default();
		layer_filled.fill(true);
		indirect.layer_yx_mut(13).combine(&layer_filled);

		verify_masks_equal(&direct, &indirect);
	}
}