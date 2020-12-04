mod compute;

use bit_vec::BitVec;
use std::cmp;
use std::ops::{Index, IndexMut};
use vocs::component::LayerStorage;
use vocs::mask::{BitLayer, Mask};
use vocs::nibbles::{u4, NibbleLayer};
use vocs::packed::PackedCube;
use vocs::position::{CubePosition, LayerPosition};

pub use compute::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CubeHeightMap {
	heights: NibbleLayer,
	is_filled: BitLayer,
}

impl CubeHeightMap {
	pub fn build(chunk: &PackedCube, matches: &BitVec, mut is_filled: BitLayer) -> Self {
		// If there are no blocks in this chunk that would match our predicate, then there is no
		// point in scanning each individual block to check it
		if matches.is_empty() || !matches.iter().any(|entry| entry) {
			return CubeHeightMap { heights: NibbleLayer::default(), is_filled };
		}

		// Check the top layer of the chunk to see which positions are filled
		for position in LayerPosition::enumerate() {
			let chunk_position = CubePosition::from_layer(15, position);
			let matches = matches.get(chunk.get(chunk_position) as usize).unwrap();

			is_filled.set_or(position, matches);
		}

		// If the height map is already full (ie, there is already a matching block in the top
		// layer of this chunk or in a chunk above this one for each horizontal position), then
		// there is no point in performing any further computations
		if is_filled.is_filled(true) {
			return CubeHeightMap { heights: NibbleLayer::default(), is_filled };
		}

		// Scan each horizontal stack of blocks within the chunk top-down to find the top matching
		// block at each horizontal position
		let mut heights = NibbleLayer::default();

		for layer in LayerPosition::enumerate() {
			if is_filled[layer] {
				continue;
			}

			// Traverse top-down so that we can bail out early
			for y in (0..15).rev() {
				let position = CubePosition::from_layer(y, layer);

				if matches.get(chunk.get(position) as usize).unwrap() {
					heights.set(position.layer(), u4::new(y + 1));

					break;
				}
			}
		}

		CubeHeightMap { heights, is_filled }
	}

	pub fn heightmap(&self) -> &NibbleLayer {
		&self.heights
	}

	pub fn is_filled(&self) -> &BitLayer {
		&self.is_filled
	}

	pub fn into_mask(mut self) -> BitLayer {
		for position in LayerPosition::enumerate() {
			let height = self.heights.get(position);

			self.is_filled.set_or(position, height != u4::new(0));
		}

		self.is_filled
	}
}

pub struct ColumnHeightMap {
	heights: Box<[u32; 256]>,
}

impl ColumnHeightMap {
	fn new() -> Self {
		ColumnHeightMap { heights: Box::new([0; 256]) }
	}

	pub fn slice(&self, chunk_y: u4) -> CubeHeightMap {
		let mut sliced =
			CubeHeightMap { heights: NibbleLayer::default(), is_filled: BitLayer::default() };

		let base = chunk_y.raw() as u32 * 16;

		for position in LayerPosition::enumerate() {
			let full_height = self[position];

			if full_height < base {
				continue;
			}

			let height = cmp::min(full_height - base, 16);

			sliced.heights.set(position, u4::new((height & 15) as u8));
			sliced.is_filled.set(position, (height & 16) == 16);
		}

		sliced
	}

	pub fn into_inner(self) -> Box<[u32; 256]> {
		self.heights
	}

	pub fn as_inner(&self) -> &[u32; 256] {
		&self.heights
	}
}

impl Index<LayerPosition> for ColumnHeightMap {
	type Output = u32;

	fn index(&self, index: LayerPosition) -> &u32 {
		&self.heights[index.zx() as usize]
	}
}

impl IndexMut<LayerPosition> for ColumnHeightMap {
	fn index_mut(&mut self, index: LayerPosition) -> &mut u32 {
		&mut self.heights[index.zx() as usize]
	}
}

pub struct HeightMapBuilder {
	heightmap: ColumnHeightMap,
	chunk_y: u8,
}

impl HeightMapBuilder {
	pub fn new() -> Self {
		HeightMapBuilder { heightmap: ColumnHeightMap::new(), chunk_y: 15 }
	}

	pub fn add(&mut self, slice: CubeHeightMap) -> BitLayer {
		assert_ne!(
			self.chunk_y, 255,
			"Tried to add too many CubeHeightMap slices to HeightMapBuilder"
		);

		let base = (self.chunk_y as u32) * 16;

		for position in LayerPosition::enumerate() {
			let height = &mut self.heightmap[position];
			let chunk_height = slice.heights.get(position);

			if *height != 0 {
				continue;
			}

			if slice.is_filled[position] {
				*height = base + 16;
			} else if chunk_height != u4::new(0) {
				*height = base + (chunk_height.raw() as u32);
			}
		}

		if self.chunk_y > 0 {
			self.chunk_y -= 1;
		} else {
			self.chunk_y = 255;
		}

		slice.into_mask()
	}

	pub fn build(self) -> ColumnHeightMap {
		assert_eq!(
			self.chunk_y, 255,
			"HeightMapBuilder::build called before all CubeHeightMap slices were provided"
		);

		self.heightmap
	}
}
