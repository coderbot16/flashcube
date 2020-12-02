use bit_vec::BitVec;

use crate::heightmap::{ChunkHeightMap, ColumnHeightMap, HeightMapBuilder};

use rayon::iter::ParallelBridge;
use rayon::prelude::ParallelIterator;

use std::collections::HashMap;

use vocs::indexed::{ChunkIndexed, Target};
use vocs::mask::LayerMask;
use vocs::position::{GlobalSectorPosition, LayerPosition};
use vocs::unpacked::Layer;
use vocs::world::sector::Sector;
use vocs::world::world::World;

pub fn compute_world_heightmaps<'a, B, F>(
	blocks: &'a World<ChunkIndexed<B>>, predicate: &'a F,
) -> HashMap<GlobalSectorPosition, Layer<ColumnHeightMap>>
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> bool + Sync,
{
	let compute_sector_heightmaps =
		|(&position, sector)| (position, compute_sector_heightmaps(sector, predicate));

	// TODO: Remove need for par_bridge
	blocks.sectors().par_bridge().map(compute_sector_heightmaps).collect()
}

pub fn compute_sector_heightmaps<'a, B, F>(
	blocks: &'a Sector<ChunkIndexed<B>>, predicate: &'a F,
) -> Layer<ColumnHeightMap>
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> bool + Sync,
{
	let compute_column_heightmap =
		|(position, column)| (position, compute_column_heightmap(column, predicate));

	// TODO: Remove need for par_bridge
	let unordered_heightmaps: Vec<(LayerPosition, ColumnHeightMap)> =
		blocks.enumerate_columns().par_bridge().map(compute_column_heightmap).collect();

	// We've received an unordered list of heightmaps from the parallel iterator.
	// It's necessary to properly sort them before returning.
	// First, we order them with the ordered_heightmaps layer...
	let mut ordered_heightmaps: Layer<Option<ColumnHeightMap>> = Layer::default();

	for (position, heightmap) in unordered_heightmaps {
		ordered_heightmaps[position] = Some(heightmap);
	}

	// ... then, we unwrap all of the heightmaps, since at this point every slot should
	// be occupied by a Some value.
	ordered_heightmaps.map(Option::unwrap)
}

pub fn compute_column_heightmap<'a, B, F>(
	column: [Option<&'a ChunkIndexed<B>>; 16], predicate: &'a F,
) -> ColumnHeightMap
where
	B: 'a + Target + Send + Sync,
	F: Fn(&'a B) -> bool,
{
	let mut mask = LayerMask::default();
	let mut heightmap_builder = HeightMapBuilder::new();

	for chunk in column.iter().rev() {
		let (blocks, palette) = match chunk {
			&Some(chunk) => chunk.freeze(),
			&None => todo!("Cannot handle empty chunks!"),
		};

		// TODO: Don't ignore corrupted chunks by silently defaulting empty palette entries
		let predicate_palette: BitVec =
			palette.iter().map(|block| block.as_ref().map(predicate).unwrap_or(false)).collect();

		let chunk_heightmap = ChunkHeightMap::build(blocks, &predicate_palette, mask);
		mask = heightmap_builder.add(chunk_heightmap);
	}

	heightmap_builder.build()
}
