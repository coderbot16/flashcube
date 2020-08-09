use cgmath::{Vector2, Vector3};
use i73_base::{math, Block, Layer, Pass};
use i73_shape::volume::{self, TriNoiseSettings, TriNoiseSource};
use java_rand::Random;
use vocs::position::{ColumnPosition, GlobalColumnPosition};
use vocs::view::ColumnMut;

pub fn default_tri_settings() -> TriNoiseSettings {
	TriNoiseSettings {
		main_out_scale: 20.0,
		upper_out_scale: 512.0,
		lower_out_scale: 512.0,
		lower_scale: Vector3::new(1368.824, 684.412, 1368.824),
		upper_scale: Vector3::new(1368.824, 684.412, 1368.824),
		main_scale: Vector3::new(1368.824 / 80.0, 684.412 / 160.0, 1368.824 / 80.0),
		y_size: 33,
	}
}

pub fn passes(seed: u64, tri_settings: &TriNoiseSettings, blocks: ShapeBlocks) -> ShapePass {
	let mut rng = Random::new(seed);

	let tri = TriNoiseSource::new(&mut rng, tri_settings);

	ShapePass { blocks, tri }
}

pub struct ShapeBlocks {
	pub solid: Block,
	pub air: Block,
}

impl Default for ShapeBlocks {
	fn default() -> Self {
		ShapeBlocks { solid: Block::STONE, air: Block::AIR }
	}
}

pub struct ShapePass {
	blocks: ShapeBlocks,
	tri: TriNoiseSource,
}

impl Pass<()> for ShapePass {
	fn apply(&self, target: &mut ColumnMut<Block>, _: &Layer<()>, chunk: GlobalColumnPosition) {
		let offset = Vector2::new((chunk.x() as f64) * 2.0, (chunk.z() as f64) * 2.0);

		let mut field = [[[0f64; 3]; 33]; 3];

		for x in 0..3 {
			for z in 0..3 {
				for y in 0..33 {
					let mut value = self.tri.sample(
						Vector3::new(offset.x + x as f64, y as f64, offset.y + z as f64),
						y,
					) - 8.0;

					value = volume::reduce_upper(value, -30.0, y as f64, 32.0, 33.0);
					value = volume::reduce_lower(value, -30.0, y as f64, 8.0);

					field[x][y][z] = value;
				}
			}
		}

		target.ensure_available(self.blocks.air.clone());
		target.ensure_available(self.blocks.solid.clone());

		let (mut blocks, palette) = target.freeze_palette();

		let air = palette.reverse_lookup(&self.blocks.air).unwrap();
		let solid = palette.reverse_lookup(&self.blocks.solid).unwrap();

		for i in 0..32768 {
			let position = ColumnPosition::from_yzx(i);

			let block = if trilinear128(&field, position) > 0.0 { &solid } else { &air };

			blocks.set(position, block);
		}
	}
}

pub fn trilinear128(array: &[[[f64; 3]; 33]; 3], position: ColumnPosition) -> f64 {
	debug_assert!(position.y() < 128, "trilinear128 only supports Y values below 128");

	let inner = (
		((position.x() % 8) as f64) / 8.0,
		((position.y() % 4) as f64) / 4.0,
		((position.z() % 8) as f64) / 8.0,
	);

	let indices =
		((position.x() / 8) as usize, (position.y() / 4) as usize, (position.z() / 8) as usize);

	math::lerp(
		math::lerp(
			math::lerp(
				array[indices.0][indices.1][indices.2],
				array[indices.0][indices.1 + 1][indices.2],
				inner.1,
			),
			math::lerp(
				array[indices.0 + 1][indices.1][indices.2],
				array[indices.0 + 1][indices.1 + 1][indices.2],
				inner.1,
			),
			inner.0,
		),
		math::lerp(
			math::lerp(
				array[indices.0][indices.1][indices.2 + 1],
				array[indices.0][indices.1 + 1][indices.2 + 1],
				inner.1,
			),
			math::lerp(
				array[indices.0 + 1][indices.1][indices.2 + 1],
				array[indices.0 + 1][indices.1 + 1][indices.2 + 1],
				inner.1,
			),
			inner.0,
		),
		inner.2,
	)
}
