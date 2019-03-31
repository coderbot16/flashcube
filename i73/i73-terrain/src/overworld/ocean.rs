use cgmath::{Point2, Vector2, Vector3};
use i73_noise::sample::Sample;
use i73_biome::climate::ClimateSource;
use i73_shape::height::HeightSource;
use i73_shape::volume::{TriNoiseSource, FieldSettings, trilinear128};
use i73_base::Pass;
use vocs::position::{ChunkPosition, GlobalColumnPosition};
use vocs::view::ColumnMut;
use i73_base::Block;
use i73_shape::height::lerp_to_layer;

pub struct OceanBlocks {
	pub air:   Block,
	pub ocean: Block
}

impl Default for OceanBlocks {
	fn default() -> Self {
		OceanBlocks {
			air:   Block::air(),
			ocean: Block::from_anvil_id( 9 * 16)
		}
	}
}

// TODO: Ice
pub struct OceanPass {
	pub climate: ClimateSource,
	pub blocks:  OceanBlocks,
	pub sea_top: usize
}

impl Pass for OceanPass {
	fn apply(&self, target: &mut ColumnMut<Block>, _: GlobalColumnPosition) {
		let chunk_base = self.sea_top / 16;

		for chunk in target.0.iter_mut().take(chunk_base) {
			chunk.replace(&self.blocks.air, self.blocks.ocean);
		}

		if chunk_base > 15 {
			return
		}

		let chunk = &mut target.0[chunk_base];

		// Check if chunk has air at all!
		if chunk.palette().reverse_lookup(&self.blocks.air).is_none() {
			return
		}

		chunk.ensure_available(self.blocks.ocean);

		let (chunk, palette) = chunk.freeze_palette();
		let ocean = palette.reverse_lookup(&self.blocks.ocean).unwrap();
		let air = palette.reverse_lookup(&self.blocks.air).unwrap();

		// Calculate how many layers of the chunk will have ocean.
		let sea_layers = self.sea_top % 16;

		for index in 0..((sea_layers*256) as u16) {
			let position = ChunkPosition::from_yzx(index);

			if chunk.get(position) == air {
				chunk.set(position, ocean);
			}
		}
	}
}