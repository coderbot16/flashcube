extern crate i73_base;
extern crate i73_trig;
extern crate java_rand;
extern crate vocs;

pub mod caves;

use i73_base::{Layer, Pass};
use i73_base::block::Block;
use java_rand::Random;
use vocs::position::GlobalColumnPosition;
use vocs::view::ColumnMut;

pub struct StructureGenerateNearby<T>
where
	T: StructureGenerator,
{
	seed_coefficients: (i64, i64),
	radius: u32,
	diameter: u32,
	world_seed: u64,
	generator: T,
}

impl<T> StructureGenerateNearby<T>
where
	T: StructureGenerator,
{
	pub fn new(world_seed: u64, radius: u32, generator: T) -> Self {
		let mut rng = Random::new(world_seed);

		StructureGenerateNearby {
			seed_coefficients: (((rng.next_i64() >> 1) << 1) + 1, ((rng.next_i64() >> 1) << 1) + 1),
			radius,
			diameter: radius * 2,
			world_seed,
			generator,
		}
	}
}

impl<T> Pass<()> for StructureGenerateNearby<T>
where
	T: StructureGenerator,
{
	fn apply(&self, target: &mut ColumnMut<Block>, _: &Layer<()>, chunk: GlobalColumnPosition) {
		let radius = self.radius as i32;

		for x in (0..self.diameter).map(|x| chunk.x() + (x as i32) - radius) {
			for z in (0..self.diameter).map(|z| chunk.z() + (z as i32) - radius) {
				let x_part = (x as i64).wrapping_mul(self.seed_coefficients.0) as u64;
				let z_part = (z as i64).wrapping_mul(self.seed_coefficients.1) as u64;

				let seed = (x_part.wrapping_add(z_part)) ^ self.world_seed;
				let from = GlobalColumnPosition::new(x, z);

				self.generator.generate(Random::new(seed), target, chunk, from, self.radius);
			}
		}
	}
}

pub trait StructureGenerator {
	fn generate(
		&self, random: Random, column: &mut ColumnMut<Block>, chunk_pos: GlobalColumnPosition,
		from: GlobalColumnPosition, radius: u32,
	);
}
