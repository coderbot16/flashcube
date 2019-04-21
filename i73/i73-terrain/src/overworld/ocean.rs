use i73_biome::climate::Climate;
use i73_base::{Pass, Layer};
use vocs::position::{ChunkPosition, GlobalColumnPosition, LayerPosition};
use vocs::view::ColumnMut;
use i73_base::Block;
use vocs::component::LayerStorage;

pub struct OceanBlocks {
	pub air:   Block,
	pub ocean: Block,
	pub ice:   Block
}

impl Default for OceanBlocks {
	fn default() -> Self {
		OceanBlocks {
			air:   Block::air(),
			ocean: Block::from_anvil_id( 9 * 16),
			ice:   Block::from_anvil_id(79 * 16)
		}
	}
}

// TODO: Ice
pub struct OceanPass {
	pub blocks:  OceanBlocks,
	pub sea_top: usize
}

impl Pass<Climate> for OceanPass {
	fn apply(&self, target: &mut ColumnMut<Block>, climates: &Layer<Climate>, column_position: GlobalColumnPosition) {
		if self.sea_top == 0 {
			return;
		}

		//let ice_mask = self.climate.freezing_layer((column_position.x() as f64, column_position.z() as f64));
		//let has_ice = !ice_mask.is_filled(false);
		let has_ice = false;

		let chunk_base = self.sea_top / 16;

		for chunk in target.0.iter_mut().take(chunk_base) {
			chunk.replace(&self.blocks.air, self.blocks.ocean);
		}

		// Make chunk_base lower because the top layer might be ice.
		let chunk_base = (self.sea_top - has_ice as usize) / 16;
		if chunk_base > 15 {
			return
		}

		let chunk = &mut target.0[chunk_base];

		// Check if chunk has air at all!
		if chunk.palette().reverse_lookup(&self.blocks.air).is_none() {
			return
		}

		chunk.ensure_available(self.blocks.ocean);
		chunk.ensure_available(self.blocks.ice);

		let (chunk, palette) = chunk.freeze_palette();
		let ocean = palette.reverse_lookup(&self.blocks.ocean).unwrap();
		let ice = palette.reverse_lookup(&self.blocks.ice).unwrap();
		let air = palette.reverse_lookup(&self.blocks.air).unwrap();

		// Calculate how many layers of the chunk will have ocean.
		let sea_layers = (self.sea_top - has_ice as usize) % 16;

		for index in 0..((sea_layers*256) as u16) {
			let position = ChunkPosition::from_yzx(index);

			if chunk.get(position) == air {
				chunk.set(position, ocean);
			}
		}

		/*if has_ice {
			println!("Ice detected! {}", column_position);
			let y = ((self.sea_top - 1) % 16) as u8;

			for layer_position in LayerPosition::enumerate() {
				let position = ChunkPosition::from_layer(y, layer_position);

				if ice_mask[layer_position] && chunk.get(position) == air {
					chunk.set(position, ice);
				}
			}
		}*/
	}
}