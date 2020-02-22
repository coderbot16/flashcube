//! Types for matching against specific block.
//! TODO: Replace with sparse bit array in `vocs`.
//! Generic types are not configurable and are a band aid.
//! A component-based solution, in comparison, would be much more configurable.
use crate::Block;
use fxhash::FxHashSet;
use std::iter::{FromIterator, IntoIterator, Iterator};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockMatcher {
	pub blocks: FxHashSet<Block>,
	pub blacklist: bool,
}

impl BlockMatcher {
	pub fn all() -> Self {
		BlockMatcher { blocks: FxHashSet::default(), blacklist: true }
	}

	pub fn none() -> Self {
		BlockMatcher { blocks: FxHashSet::default(), blacklist: false }
	}

	pub fn is(block: Block) -> Self {
		let mut blocks = FxHashSet::default();
		blocks.insert(block);

		BlockMatcher { blocks, blacklist: false }
	}

	pub fn is_not(block: Block) -> Self {
		let mut blocks = FxHashSet::default();
		blocks.insert(block);

		BlockMatcher { blocks, blacklist: true }
	}

	pub fn include<'a, I>(blocks: I) -> Self
	where
		I: IntoIterator<Item = &'a Block>,
	{
		BlockMatcher { blocks: FxHashSet::from_iter(blocks.into_iter().cloned()), blacklist: false }
	}

	pub fn exclude<'a, I>(blocks: I) -> Self
	where
		I: IntoIterator<Item = &'a Block>,
	{
		BlockMatcher { blocks: FxHashSet::from_iter(blocks.into_iter().cloned()), blacklist: true }
	}

	pub fn matches(&self, block: &Block) -> bool {
		// NotPresent, Whitelist => 0 ^ 0 => 0
		// NotPresent, Blacklist => 0 ^ 1 => 1
		// Contains, Whitelist => 1 ^ 0 => 1
		// Contains, Blacklist => 1 ^ 1 => 0
		self.blocks.contains(block) ^ self.blacklist
	}
}
