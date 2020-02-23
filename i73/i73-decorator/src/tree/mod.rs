mod large;
mod normal;

use i73_base::matcher::BlockMatcher;
use i73_base::Block;
pub use large::LargeTreeDecorator;
pub use normal::NormalTreeDecorator;
use vocs::position::QuadPosition;
use vocs::view::{QuadBlocks, QuadAssociation, QuadPalettes};
use std::i32;

struct FoliageLayer {
	position: QuadPosition,
	radius: u8
}

impl FoliageLayer {
	fn place_corners<F>(
		&self, blocks: &mut QuadBlocks, foliage: &QuadAssociation, palette: &QuadPalettes<Block>, replace: &BlockMatcher,
		corner_predicate: F
	) where F: FnOnce(u8) -> bool {

		// -Z,-X
		// -Z,+X
		// +Z,-X
		// +Z,+X



		for z_offset in -radius..=radius {
			for x_offset in -radius..=radius {
				if i32::abs(z_offset) == radius && i32::abs(x_offset) == radius {
					// Predicate:
					// rng.next_u32_bound(self.settings.foliage_corner_chance) != 0 && y < tree.trunk_top
					if !corner_predicate(self.position.y()) {
						continue;
					}

					let position = match position.offset((x_offset as i8, 0, z_offset as i8)) {
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

	fn place(
		&self, blocks: &mut QuadBlocks, foliage: &QuadAssociation, palette: &QuadPalettes<Block>, replace: &BlockMatcher,
	) {
		for z_offset in -radius..=radius {
			for x_offset in -radius..=radius {
				if i32::abs(z_offset) == radius && i32::abs(x_offset) == radius {
					continue;
				}

				let position = match position.offset((x_offset as i8, 0, z_offset as i8)) {
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
			log: Block::from_anvil_id(17 * 16),
			foliage: Block::from_anvil_id(18 * 16),
			replace: BlockMatcher::include(&[Block::air(), Block::from_anvil_id(18 * 16)]),
			soil: BlockMatcher::include(
				&[Block::from_anvil_id(2 * 16), Block::from_anvil_id(3 * 16)],
			),
			new_soil: Block::from_anvil_id(3 * 16),
		}
	}
}
