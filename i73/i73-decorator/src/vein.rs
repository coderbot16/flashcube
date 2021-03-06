use crate::{Decorator, Result};
use i73_base::matcher::BlockMatcher;
use i73_base::math;
use i73_base::block::Block;
use i73_trig as trig;
use java_rand::Random;
use vocs::position::{Offset, QuadPosition};
use vocs::view::QuadMut;

// TODO: Is this really 3.141593?
/// For when you don't have the time to type out all the digits of π or Math.PI.
const NOTCHIAN_PI: f32 = 3.1415927;

/// The radius is in the range `[0.0, 0.5+size/RADIUS_DIVISOR]`
const RADIUS_DIVISOR: f64 = 16.0;
/// The length is `size/LENGTH_DIVISOR`
const LENGTH_DIVISOR: f32 = 8.0;

#[derive(Debug, Clone)]
pub struct SeasideVeinDecorator {
	pub vein: VeinDecorator,
	pub ocean: BlockMatcher,
}

impl Decorator for SeasideVeinDecorator {
	fn generate(
		&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition,
	) -> Result {
		if !self.ocean.matches(quad.get(position.offset((-8, 0, -8)).unwrap())) {
			return Ok(());
		}

		self.vein.generate(quad, rng, position)
	}
}

#[derive(Debug, Clone)]
pub struct VeinDecorator {
	pub blocks: VeinBlocks,
	pub size: u32,
}

impl Decorator for VeinDecorator {
	fn generate(
		&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition,
	) -> Result {
		let vein = Vein::create(
			self.size,
			(position.x() as i32, position.y() as i32, position.z() as i32),
			rng,
		);
		self.blocks.generate(&vein, quad, rng)
	}
}

#[derive(Debug, Clone)]
pub struct VeinBlocks {
	pub replace: BlockMatcher,
	pub block: Block,
}

impl VeinBlocks {
	pub fn generate(&self, vein: &Vein, quad: &mut QuadMut<Block>, rng: &mut Random) -> Result {
		quad.ensure_available(self.block.clone());

		let (mut blocks, palette) = quad.freeze_palette();

		let block = palette.reverse_lookup(&self.block).unwrap();

		for index in 0..(vein.size + 1) {
			let spheroid = vein.spheroid(index, rng);

			for y in spheroid.lower.1..(spheroid.upper.1 + 1) {
				for z in spheroid.lower.2..(spheroid.upper.2 + 1) {
					for x in spheroid.lower.2..(spheroid.upper.2 + 1) {
						let at = QuadPosition::new(x as u8, y as u8, z as u8); // TODO

						if spheroid.distance_squared((x, y, z)) < 1.0
							&& self.replace.matches(blocks.get(at, &palette))
						{
							blocks.set(at, &block);
						}
					}
				}
			}
		}

		Ok(())
	}
}

#[derive(Debug)]
pub struct Vein {
	/// Size of the vein. Controls iterations, radius of the spheroids, and length of the line.
	size: u32,
	/// Size as a f64, to avoid excessive casting.
	size_f64: f64,
	/// Size as a f32, to avoid excessive casting.
	size_f32: f32,
	/// Start point of the line, but not neccesarily the minimum on the Y axis.
	from: (f64, f64, f64),
	/// End point of the line, but not neccesarily the maximum on the Y axis.
	to: (f64, f64, f64),
}

impl Vein {
	pub fn create(size: u32, base: (i32, i32, i32), rng: &mut Random) -> Self {
		let size_f32 = size as f32;

		let angle = rng.next_f32() * NOTCHIAN_PI;
		let x_size = trig::sin(angle) * size_f32 / LENGTH_DIVISOR;
		let z_size = trig::cos(angle) * size_f32 / LENGTH_DIVISOR;

		let from = (
			(base.0 as f32 + x_size) as f64,
			(base.1 + 2 + rng.next_i32_bound(3)) as f64,
			(base.2 as f32 + z_size) as f64,
		);

		let to = (
			(base.0 as f32 - x_size) as f64,
			(base.1 + 2 + rng.next_i32_bound(3)) as f64,
			(base.2 as f32 - z_size) as f64,
		);

		Vein { size, size_f64: size as f64, size_f32, from, to }
	}

	pub fn spheroid(&self, index: u32, rng: &mut Random) -> Spheroid {
		let index_f64 = index as f64;
		let index_f32 = index as f32;

		let center = (
			math::lerp_fraction(self.from.0, self.to.0, index_f64, self.size_f64),
			math::lerp_fraction(self.from.1, self.to.1, index_f64, self.size_f64),
			math::lerp_fraction(self.from.2, self.to.2, index_f64, self.size_f64),
		);

		let radius_multiplier = rng.next_f64() * self.size_f64 / RADIUS_DIVISOR;

		// The sin function varies the diameter over time, so that larger diameters are closer to the center.
		let diameter = (trig::sin(index_f32 * NOTCHIAN_PI / self.size_f32) + 1.0f32) as f64
			* radius_multiplier
			+ 1.0;
		let radius = diameter / 2.0;

		// TODO: i32 casts can overflow.
		let lower = (
			(center.0 - radius).floor() as i32,
			(center.1 - radius).floor() as i32,
			(center.2 - radius).floor() as i32,
		);

		let upper = (
			(center.0 + radius).floor() as i32,
			(center.1 + radius).floor() as i32,
			(center.2 + radius).floor() as i32,
		);

		Spheroid { center, radius, lower, upper }
	}
}

#[derive(Debug)]
pub struct Spheroid {
	center: (f64, f64, f64),
	radius: f64,
	lower: (i32, i32, i32),
	upper: (i32, i32, i32),
}

impl Spheroid {
	pub fn distance_squared(&self, at: (i32, i32, i32)) -> f64 {
		let dist_x_sq = ((at.0 as f64 + 0.5 - self.center.0) / self.radius).powi(2);
		let dist_y_sq = ((at.1 as f64 + 0.5 - self.center.1) / self.radius).powi(2);
		let dist_z_sq = ((at.2 as f64 + 0.5 - self.center.2) / self.radius).powi(2);

		dist_x_sq + dist_y_sq + dist_z_sq
	}
}
