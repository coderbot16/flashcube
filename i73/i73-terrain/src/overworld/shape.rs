use cgmath::{Point2, Vector2, Vector3};
use i73_noise::sample::Sample;
use i73_biome::climate::ClimateSource;
use i73_shape::height::HeightSource;
use i73_shape::volume::{TriNoiseSource, ShapeSettings, trilinear128};
use i73_base::Pass;
use vocs::position::{ColumnPosition, GlobalColumnPosition};
use vocs::view::ColumnMut;
use i73_base::Block;
use i73_shape::height::lerp_to_layer;

pub struct ShapeBlocks {
	pub solid: Block,
	pub air:   Block
}

impl Default for ShapeBlocks {
	fn default() -> Self {
		ShapeBlocks {
			solid: Block::from_anvil_id( 1 * 16),
			air:   Block::air()
		}
	}
}

pub struct ShapePass {
	pub climate: ClimateSource,
	pub blocks:  ShapeBlocks,
	pub tri:     TriNoiseSource,
	pub height:  HeightSource,
	pub field:   ShapeSettings
}

impl Pass for ShapePass {
	fn apply(&self, target: &mut ColumnMut<Block>, chunk: GlobalColumnPosition) {
		let offset = Point2::new(
			(chunk.x() as f64) * 4.0,
			(chunk.z() as f64) * 4.0
		);

		let block_offset = (
			(chunk.x() as f64) * 16.0,
			(chunk.z() as f64) * 16.0
		);

		let climate_chunk = self.climate.chunk(block_offset);

		let mut field = [[[0f64; 5]; 17]; 5];

		for x in 0..5 {
			for z in 0..5 {
				let layer = lerp_to_layer(Vector2::new(x as u8, z as u8));

				let climate = climate_chunk.get(layer);
				let height = self.height.sample(offset + Vector2::new(x as f64, z as f64), climate);

				for y in 0..17 {
					let tri = self.tri.sample(Vector3::new(offset.x + x as f64, y as f64, offset.y + z as f64), y);

					field[x][y][z] = self.field.compute_noise_value(y as f64, height, tri);
				}
			}
		}

		target.ensure_available(self.blocks.air.clone());
		target.ensure_available(self.blocks.solid.clone());

		let (mut blocks, palette) = target.freeze_palette();

		let air   = palette.reverse_lookup(&self.blocks.air).unwrap();
		let solid = palette.reverse_lookup(&self.blocks.solid).unwrap();

		for i in 0..32768 {
			let position = ColumnPosition::from_yzx(i);

			let block = if trilinear128(&field, position) > 0.0 {
				&solid
			} else {
				&air
			};

			blocks.set(position, block);
		}
	}
}