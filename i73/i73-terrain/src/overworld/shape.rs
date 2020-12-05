use cgmath::{Point2, Vector2, Vector3};
use i73_base::math;
use i73_base::block::{self, Block};
use i73_base::Pass;
use i73_biome::climate::Climate;
use i73_shape::height::lerp_to_layer;
use i73_shape::height::HeightSource;
use i73_shape::volume::{ShapeSettings, TriNoiseSource};
use vocs::position::{CubePosition, GlobalColumnPosition};
use vocs::view::ColumnMut;
use vocs::unpacked::Layer;

pub struct ShapeBlocks {
	pub solid: Block,
	pub air: Block,
}

impl Default for ShapeBlocks {
	fn default() -> Self {
		ShapeBlocks { solid: block::STONE, air: block::AIR }
	}
}

pub struct ShapePass {
	pub blocks: ShapeBlocks,
	pub tri: TriNoiseSource,
	pub height: HeightSource,
	pub field: ShapeSettings,
}

impl Pass<Climate> for ShapePass {
	fn apply(
		&self, target: &mut ColumnMut<Block>, climates: &Layer<Climate>,
		chunk: GlobalColumnPosition,
	) {
		let offset = Point2::new((chunk.x() as f64) * 4.0, (chunk.z() as f64) * 4.0);

		let mut field = [[[0f64; 5]; 5]; 17];

		for x in 0..5 {
			for z in 0..5 {
				let layer = lerp_to_layer(Vector2::new(x as u8, z as u8));

				let climate = climates[layer];
				let height = self.height.sample(offset + Vector2::new(x as f64, z as f64), climate);

				for y in 0..17 {
					let tri = self.tri.sample(
						Vector3::new(offset.x + x as f64, y as f64, offset.y + z as f64),
						y,
					);

					field[y][z][x] = self.field.compute_noise_value(y as f64, height, tri);
				}
			}
		}

		for (index, chunk) in target.0.iter_mut().enumerate().take(8) {
			let section: &[[[f64; 5]; 5]; 3] = array_ref!(field, index * 2, 3);

			if let Some(fill) = is_filled(&section) {
				if fill {
					chunk.fill(self.blocks.solid.clone());
				} else {
					chunk.fill(self.blocks.air.clone());
				}

				continue;
			}

			chunk.ensure_available(self.blocks.air.clone());
			chunk.ensure_available(self.blocks.solid.clone());

			let (blocks, palette) = chunk.freeze_palette();

			let air = palette.reverse_lookup(&self.blocks.air).unwrap();
			let solid = palette.reverse_lookup(&self.blocks.solid).unwrap();

			for position in CubePosition::enumerate() {
				let block = if trilinear(&section, position) > 0.0 { solid } else { air };

				blocks.set(position, block);
			}
		}
	}
}

pub fn is_filled(array: &[[[f64; 5]; 5]; 3]) -> Option<bool> {
	let mut empty = true;
	let mut full = true;

	for y in 0..3 {
		for z in 0..5 {
			for x in 0..5 {
				if array[y][z][x] > 0.0 {
					empty = false;
				} else {
					full = false;
				}
			}
		}
	}

	if empty {
		Some(false)
	} else if full {
		Some(true)
	} else {
		None
	}
}

pub fn trilinear(array: &[[[f64; 5]; 5]; 3], position: CubePosition) -> f64 {
	let inner = (
		((position.y() % 8) as f64) / 8.0,
		((position.z() % 4) as f64) / 4.0,
		((position.x() % 4) as f64) / 4.0,
	);

	let indices =
		((position.y() / 8) as usize, (position.z() / 4) as usize, (position.x() / 4) as usize);

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
