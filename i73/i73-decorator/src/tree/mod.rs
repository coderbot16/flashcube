mod large;
mod normal;

use i73_base::matcher::BlockMatcher;
use i73_base::block::{self, Block};
pub use large::LargeTreeDecorator;
pub use normal::NormalTreeDecorator;
use std::i32;
use vocs::position::{Offset, QuadPosition};
use vocs::view::{QuadAssociation, QuadBlocks, QuadPalettes};

struct FoliageLayer {
	position: QuadPosition,
	radius: u8,
}

impl FoliageLayer {
	fn place_corners<F>(
		&self, blocks: &mut QuadBlocks, foliage: &QuadAssociation, palette: &QuadPalettes<Block>,
		replace: &BlockMatcher, mut corner_predicate: F,
	) where
		F: FnMut(u8) -> bool,
	{
		let mut try_corner = |x_offset, z_offset| {
			if !corner_predicate(self.position.y()) {
				return;
			}

			let position = match self.position.offset((x_offset, 0i8, z_offset)) {
				Some(position) => position,
				None => return,
			};

			if replace.matches(blocks.get(position, palette)) {
				blocks.set(position, foliage);
			}
		};

		let radius = self.radius as i8;

		if radius == 0 {
			try_corner(0, 0);
		} else {
			try_corner(-radius, -radius);
			try_corner(radius, -radius);
			try_corner(-radius, radius);
			try_corner(radius, radius);
		}
	}

	fn place(
		&self, blocks: &mut QuadBlocks, foliage: &QuadAssociation, palette: &QuadPalettes<Block>,
		replace: &BlockMatcher,
	) {
		let radius = self.radius as i32;

		for z_offset in -radius..=radius {
			for x_offset in -radius..=radius {
				if i32::abs(z_offset) == radius && i32::abs(x_offset) == radius {
					continue;
				}

				let position = match self.position.offset((x_offset as i8, 0i8, z_offset as i8)) {
					Some(position) => position,
					None => continue,
				};

				if replace.matches(blocks.get(position, palette)) {
					blocks.set(position, foliage);
				}
			}
		}
	}
}

struct TreeBlocks {
	log: Block,
	foliage: Block,
	replace: BlockMatcher,
	soil: BlockMatcher,
	new_soil: Block,
}

impl Default for TreeBlocks {
	fn default() -> Self {
		TreeBlocks {
			log: block::OAK_LOG,
			foliage: block::OAK_LEAVES,
			replace: BlockMatcher::include(&[block::AIR, block::OAK_LEAVES]),
			soil: BlockMatcher::include(&[block::GRASS, block::DIRT]),
			new_soil: block::DIRT,
		}
	}
}
