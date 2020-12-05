use cgmath::{Vector2, Vector3};
use i73_base::Pass;
use i73_base::block::Block;
use i73_shape::volume::{self, trilinear128, TriNoiseSettings, TriNoiseSource};
use java_rand::Random;
use vocs::position::{ColumnPosition, GlobalColumnPosition};
use vocs::view::ColumnMut;
use vocs::unpacked::Layer;

pub use crate::overworld::shape::ShapeBlocks;

const NOTCH_PI_F64: f64 = 3.1415926535897931;

pub fn default_tri_settings() -> TriNoiseSettings {
	TriNoiseSettings {
		main_out_scale: 20.0,
		upper_out_scale: 512.0,
		lower_out_scale: 512.0,
		lower_scale: Vector3::new(684.412, 2053.236, 684.412),
		upper_scale: Vector3::new(684.412, 2053.236, 684.412),
		main_scale: Vector3::new(684.412 / 80.0, 2053.236 / 60.0, 684.412 / 80.0),
		y_size: 33,
	}
}

pub fn passes(
	seed: u64, tri_settings: &TriNoiseSettings, blocks: ShapeBlocks
) -> ShapePass {
	let mut rng = Random::new(seed);

	let tri = TriNoiseSource::new(&mut rng, tri_settings);

	ShapePass { blocks, tri, reduction: generate_reduction_table(17) }
}

pub struct ShapePass {
	blocks: ShapeBlocks,
	tri: TriNoiseSource,
	reduction: Vec<f64>
}

impl Pass<()> for ShapePass {
	fn apply(&self, target: &mut ColumnMut<Block>, _: &Layer<()>, chunk: GlobalColumnPosition) {
		let offset = Vector2::new((chunk.x() as f64) * 4.0, (chunk.z() as f64) * 4.0);

		let mut field = [[[0f64; 5]; 17]; 5];

		for x in 0..5 {
			for z in 0..5 {
				for y in 0..17 {
					let mut value = self.tri.sample(
						Vector3::new(offset.x + x as f64, y as f64, offset.y + z as f64),
						y,
					);

					value -= self.reduction[y];
					value = volume::reduce_upper(value, -10.0, y as f64, 4.0, 17.0);

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

			let block = if trilinear128(&field, position) > 0.0 {
				&solid
			} else {
				&air
			};

			blocks.set(position, block);
		}
	}
}

pub fn generate_reduction_table(y_size: usize) -> Vec<f64> {
	let mut data = Vec::with_capacity(y_size);
	let y_size_f64 = y_size as f64;

	for index in 0..y_size {
		let index_f64 = index as f64;

		let mut value = ((index_f64 * NOTCH_PI_F64 * 6.0) / y_size_f64).cos() * 2.0;

		value = volume::reduce_cubic(value, y_size_f64 - 1.0 - index_f64);
		value = volume::reduce_cubic(value, index_f64);

		data.push(value);
	}

	data
}
