use crate::sample::Sample;
use crate::Permutations;
use cgmath::{Point2, Vector2, Vector3};
use i73_base::math;
use java_rand::Random;

const GRAD_TABLE: [(f64, f64, f64); 16] = [
	(1.0, 1.0, 0.0),
	(-1.0, 1.0, 0.0),
	(1.0, -1.0, 0.0),
	(-1.0, -1.0, 0.0),
	(1.0, 0.0, 1.0),
	(-1.0, 0.0, 1.0),
	(1.0, 0.0, -1.0),
	(-1.0, 0.0, -1.0),
	(0.0, 1.0, 1.0),
	(0.0, -1.0, 1.0),
	(0.0, 1.0, -1.0),
	(0.0, -1.0, -1.0),
	(1.0, 1.0, 0.0),
	(0.0, -1.0, 1.0),
	(-1.0, 1.0, 0.0),
	(0.0, -1.0, -1.0),
];

/// Returns the dot product of the vector with a pseudorandomly selected gradient vector.
fn grad(t: u16, vec: Vector3<f64>) -> f64 {
	let gradient = GRAD_TABLE[(t & 0xF) as usize];
	gradient.0 * vec.x + gradient.1 * vec.y + gradient.2 * vec.z
}

/// Convienience method to call grad with Y=0.
fn grad2d(t: u16, vec: Vector2<f64>) -> f64 {
	grad(t, Vector3::new(vec.x, 0.0, vec.y))
}

fn fade(t: f64) -> f64 {
	t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Perlin noise generator. Can be sampled in 2 or 3 dimensions.
#[derive(Debug)]
pub struct Perlin {
	p: Permutations,
	scale: Vector3<f64>,
	amplitude: f64,
}

impl Perlin {
	pub fn new(p: Permutations, scale: Vector3<f64>, amplitude: f64) -> Self {
		Perlin { p, scale, amplitude }
	}

	pub fn from_rng(rng: &mut Random, scale: Vector3<f64>, amplitude: f64) -> Self {
		Perlin { p: Permutations::new(rng), scale, amplitude }
	}

	fn hash(&self, i: u16) -> u16 {
		self.p.hash(i)
	}

	// TODO: Merge generate and generate_override.

	pub fn generate(&self, loc: Vector3<f64>) -> f64 {
		let loc = Vector3::new(loc.x * self.scale.x, loc.y * self.scale.y, loc.z * self.scale.z)
			+ self.p.offset;

		// TODO: Make sure we still get the far lands.
		let floored = loc.map(math::floor_clamped);

		// Note: Since floor_clamped converts to the range of i32, these casts are safe.
		let p = Vector3::new(
			(floored.x as i32 & 0xFF) as u16,
			(floored.y as i32 & 0xFF) as u16,
			(floored.z as i32 & 0xFF) as u16,
		);

		// Find the position of the point within the cell.
		let loc = loc - floored;

		// Use the fade function to reduce unnatural looking artifacts from direct linear interpolation.
		let faded = loc.map(fade);

		let a = self.hash(p.x) + p.y;
		let aa = self.hash(a) + p.z;
		let ab = self.hash(a + 1) + p.z;

		let b = self.hash(p.x + 1) + p.y;
		let ba = self.hash(b) + p.z;
		let bb = self.hash(b + 1) + p.z;

		math::lerp(
			math::lerp(
				math::lerp(
					grad(self.hash(aa), loc),
					grad(self.hash(ba), loc - Vector3::new(1.0, 0.0, 0.0)),
					faded.x,
				),
				math::lerp(
					grad(self.hash(ab), loc - Vector3::new(0.0, 1.0, 0.0)),
					grad(self.hash(bb), loc - Vector3::new(1.0, 1.0, 0.0)),
					faded.x,
				),
				faded.y,
			),
			math::lerp(
				math::lerp(
					grad(self.hash(aa + 1), loc - Vector3::new(0.0, 0.0, 1.0)),
					grad(self.hash(ba + 1), loc - Vector3::new(1.0, 0.0, 1.0)),
					faded.x,
				),
				math::lerp(
					grad(self.hash(ab + 1), loc - Vector3::new(0.0, 1.0, 1.0)),
					grad(self.hash(bb + 1), loc - Vector3::new(1.0, 1.0, 1.0)),
					faded.x,
				),
				faded.y,
			),
			faded.z,
		) * self.amplitude
	}

	pub fn generate_override(&self, loc: Vector3<f64>, actual_y: f64) -> f64 {
		let loc = Vector3::new(loc.x * self.scale.x, loc.y * self.scale.y, loc.z * self.scale.z)
			+ self.p.offset;

		// TODO: Make sure we still get the far lands.
		let floored = loc.map(math::floor_clamped);

		// Note: Since floor_clamped converts to the range of i32, these casts are safe.
		let p = Vector3::new(
			(floored.x as i32 & 0xFF) as u16,
			(floored.y as i32 & 0xFF) as u16,
			(floored.z as i32 & 0xFF) as u16,
		);

		let mut loc = loc - floored;
		let faded = loc.map(fade);
		loc.y = actual_y;

		let a = self.hash(p.x) + p.y;
		let aa = self.hash(a) + p.z;
		let ab = self.hash(a + 1) + p.z;

		let b = self.hash(p.x + 1) + p.y;
		let ba = self.hash(b) + p.z;
		let bb = self.hash(b + 1) + p.z;

		math::lerp(
			math::lerp(
				math::lerp(
					grad(self.hash(aa), loc),
					grad(self.hash(ba), loc - Vector3::new(1.0, 0.0, 0.0)),
					faded.x,
				),
				math::lerp(
					grad(self.hash(ab), loc - Vector3::new(0.0, 1.0, 0.0)),
					grad(self.hash(bb), loc - Vector3::new(1.0, 1.0, 0.0)),
					faded.x,
				),
				faded.y,
			),
			math::lerp(
				math::lerp(
					grad(self.hash(aa + 1), loc - Vector3::new(0.0, 0.0, 1.0)),
					grad(self.hash(ba + 1), loc - Vector3::new(1.0, 0.0, 1.0)),
					faded.x,
				),
				math::lerp(
					grad(self.hash(ab + 1), loc - Vector3::new(0.0, 1.0, 1.0)),
					grad(self.hash(bb + 1), loc - Vector3::new(1.0, 1.0, 1.0)),
					faded.x,
				),
				faded.y,
			),
			faded.z,
		) * self.amplitude
	}

	pub fn generate_y_table(&self, start: f64, table: &mut [f64]) {
		let mut actual_y = 0.0;
		let mut last_p = 65535;

		for (offset, entry) in table.iter_mut().enumerate() {
			let y = (start + (offset as f64)) * self.scale.y + self.p.offset.y;
			let floored = math::floor_clamped(y);
			let p = (floored % 256.0) as u16;
			let y = y - floored;

			if p != last_p {
				actual_y = y;
			}

			last_p = p;

			*entry = actual_y;
		}
	}
}

impl Sample for Perlin {
	type Output = f64;

	fn sample(&self, loc: Point2<f64>) -> f64 {
		let loc = Vector2::new(loc.x * self.scale.x, loc.y * self.scale.z)
			+ Vector2::new(self.p.offset.x, self.p.offset.z);

		// TODO: Verify the farlands effect.
		let floored = loc.map(math::floor_clamped);

		// Note: Since floor_clamped converts to the range of i32, these casts are safe.
		let p = Vector2::new((floored.x as i32 & 0xFF) as u16, (floored.y as i32 & 0xFF) as u16);

		let loc = loc - floored;
		let faded = loc.map(fade);

		let aa = self.hash(self.hash(p.x)) + p.y;
		let ba = self.hash(self.hash(p.x + 1)) + p.y;

		math::lerp(
			math::lerp(
				grad2d(self.hash(aa), loc),
				grad2d(self.hash(ba), loc - Vector2::new(1.0, 0.0)),
				faded.x,
			),
			math::lerp(
				grad2d(self.hash(aa + 1), loc - Vector2::new(0.0, 1.0)),
				grad2d(self.hash(ba + 1), loc - Vector2::new(1.0, 1.0)),
				faded.x,
			),
			faded.y,
		) * self.amplitude
	}
}
