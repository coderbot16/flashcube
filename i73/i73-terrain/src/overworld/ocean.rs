use i73_base::block::{self, Block};
use i73_base::Pass;
use i73_biome::climate::{self, Climate};
use vocs::component::LayerStorage;
use vocs::position::{CubePosition, GlobalColumnPosition, LayerPosition};
use vocs::view::ColumnMut;
use vocs::unpacked::Layer;

pub struct OceanBlocks {
	pub air: Block,
	pub ocean: Block,
	pub ice: Block,
}

impl Default for OceanBlocks {
	fn default() -> Self {
		OceanBlocks {
			air: block::AIR,
			ocean: block::STILL_WATER,
			ice: block::ICE,
		}
	}
}

pub struct OceanPass {
	pub blocks: OceanBlocks,
	pub sea_top: usize,
}

impl Pass<Climate> for OceanPass {
	fn apply(
		&self, target: &mut ColumnMut<Block>, climates: &Layer<Climate>, _: GlobalColumnPosition,
	) {
		if self.sea_top == 0 {
			return;
		}

		let ice_mask = climate::freezing_layer(climates);
		let has_ice = !ice_mask.is_filled(false);

		// TODO: Optimization: When the top layer of the chunk is ice, fill the rest with water.
		let chunk_base = (self.sea_top - has_ice as usize) / 16;
		for chunk in target.0.iter_mut().take(chunk_base) {
			chunk.replace(&self.blocks.air, self.blocks.ocean);
		}

		if chunk_base > 15 {
			return;
		}

		let chunk = &mut target.0[chunk_base];

		// Check if chunk has air at all!
		if chunk.palette().reverse_lookup(&self.blocks.air).is_none() {
			return;
		}

		chunk.ensure_available(self.blocks.ocean);

		if has_ice {
			chunk.ensure_available(self.blocks.ice);
		}

		let (chunk, palette) = chunk.freeze_palette();
		let ocean = palette.reverse_lookup(&self.blocks.ocean).unwrap();
		let ice =
			if has_ice { Some(palette.reverse_lookup(&self.blocks.ice).unwrap()) } else { None };
		let air = palette.reverse_lookup(&self.blocks.air).unwrap();

		// Calculate how many layers of the chunk will have ocean.
		let sea_layers = (self.sea_top - has_ice as usize) % 16;

		for index in 0..((sea_layers * 256) as u16) {
			let position = CubePosition::from_yzx(index);

			if chunk.get(position) == air {
				chunk.set(position, ocean);
			}
		}

		if let Some(ice) = ice {
			let y = ((self.sea_top - 1) % 16) as u8;

			for layer_position in LayerPosition::enumerate() {
				let position = CubePosition::from_layer(y, layer_position);

				if chunk.get(position) == air {
					chunk.set(position, if ice_mask[layer_position] { ice } else { ocean });
				}
			}
		}
	}
}
