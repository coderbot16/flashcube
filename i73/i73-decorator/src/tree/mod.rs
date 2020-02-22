mod large;
mod normal;

use i73_base::matcher::BlockMatcher;
use i73_base::Block;
pub use large::LargeTreeDecorator;
pub use normal::NormalTreeDecorator;

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
			replace: BlockMatcher::include([Block::air(), Block::from_anvil_id(18 * 16)].iter()),
			soil: BlockMatcher::include(
				[Block::from_anvil_id(2 * 16), Block::from_anvil_id(3 * 16)].iter(),
			),
			new_soil: Block::from_anvil_id(3 * 16),
		}
	}
}
