pub const AIR: Block = Block(0);
pub const STONE: Block = Block(1 * 16);
pub const GRASS: Block = Block(2 * 16);
pub const DIRT: Block = Block(3 * 16);
pub const BEDROCK: Block = Block(7 * 16);
pub const FLOWING_WATER: Block = Block(8 * 16);
pub const STILL_WATER: Block = Block(9 * 16);
pub const FLOWING_LAVA: Block = Block(10 * 16);
pub const STILL_LAVA: Block = Block(11 * 16);
pub const SAND: Block = Block(12 * 16);
pub const GRAVEL: Block = Block(13 * 16);
pub const OAK_LOG: Block = Block(17 * 16);
pub const OAK_LEAVES: Block = Block(18 * 16);
pub const SANDSTONE: Block = Block(24 * 16);
pub const TALL_GRASS: Block = Block(31 * 16 + 1);
pub const FARMLAND: Block = Block(60 * 16);
pub const ICE: Block = Block(79 * 16);
pub const CLAY: Block = Block(82 * 16);
pub const NETHERRACK: Block = Block(87 * 16);

/// ID of a block.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Block(u16);

impl Into<u16> for Block {
	fn into(self) -> u16 {
		self.0
	}
}
