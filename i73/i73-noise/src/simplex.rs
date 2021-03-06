use crate::sample::Sample;
use crate::Permutations;
use cgmath::{Point2, Vector2};
use i73_base::math;
use java_rand::Random;

const GRAD_TABLE: [(f64, f64); 12] = [
	(1.0, 1.0),
	(-1.0, 1.0),
	(1.0, -1.0),
	(-1.0, -1.0),
	(1.0, 0.0),
	(-1.0, 0.0),
	(1.0, 0.0),
	(-1.0, 0.0),
	(0.0, 1.0),
	(0.0, -1.0),
	(0.0, 1.0),
	(0.0, -1.0),
];

fn grad(hash: u16, x: f64, y: f64) -> f64 {
	let gradient = GRAD_TABLE[hash as usize % 12];
	gradient.0 * x + gradient.1 * y
}

const SQRT_THREE: f64 = 1.7320508075688772935;

const F2: f64 = 0.5 * (SQRT_THREE - 1.0);
const G2: f64 = (3.0 - SQRT_THREE) / 6.0;

// We can only implement Simplex noise up to 2D or we will run into patent issues.
#[derive(Debug, Clone)]
pub struct Simplex {
	p: Permutations,
	scale: Vector2<f64>,
	amplitude: f64,
}

impl Simplex {
	pub fn new(p: Permutations, scale: Vector2<f64>, amplitude: f64) -> Self {
		Simplex { p, scale, amplitude }
	}

	pub fn from_rng(rng: &mut Random, scale: Vector2<f64>, amplitude: f64) -> Self {
		Simplex { p: Permutations::new(rng), scale, amplitude }
	}

	fn hash(&self, i: u16) -> u16 {
		self.p.hash(i)
	}
}

impl Sample for Simplex {
	type Output = f64;

	fn sample(&self, point: Point2<f64>) -> f64 {
		let point = Point2::new(point.x * self.scale.x, point.y * self.scale.y)
			+ Vector2::new(self.p.offset.x, self.p.offset.y);

		let s = (point.x + point.y) * F2;
		let fx = math::floor_clamped(point.x + s);
		let fy = math::floor_clamped(point.y + s);
		let t = (fx + fy) * G2;

		let x0 = point.x - (fx - t);
		let y0 = point.y - (fy - t);

		let bias = if x0 > y0 { Vector2::new(1, 0) } else { Vector2::new(0, 1) };

		let x1 = x0 - (bias.x as f64) + G2;
		let y1 = y0 - (bias.y as f64) + G2;
		let x2 = x0 - 1.0 + G2 * 2.0;
		let y2 = y0 - 1.0 + G2 * 2.0;

		// TODO: This is broken for negative coords.
		let x_i = (fx as i32 & 0xFF) as u16;
		let y_i = (fy as i32 & 0xFF) as u16;

		let t0 = f64::max(0.5 - x0 * x0 - y0 * y0, 0.0);
		let n0 = f64::powi(t0, 4) * grad(self.hash(x_i + self.hash(y_i)), x0, y0);

		let t1 = f64::max(0.5 - x1 * x1 - y1 * y1, 0.0);
		let n1 = f64::powi(t1, 4) * grad(self.hash(x_i + bias.x + self.hash(y_i + bias.y)), x1, y1);

		let t2 = f64::max(0.5 - x2 * x2 - y2 * y2, 0.0);
		let n2 = f64::powi(t2, 4) * grad(self.hash(x_i + 1 + self.hash(y_i + 1)), x2, y2);

		(70.0 * self.amplitude) * (n0 + n1 + n2)
	}
}
