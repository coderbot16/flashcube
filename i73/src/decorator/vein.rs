use rng::JavaRng;
use trig::TrigLookup;

// TODO: Is this really 3.141593?
/// For when you don't have the time to type out all the digits of π or Math.PI.
const NOTCHIAN_PI: f32 = 3.1415927;

/// The radius is in the range `[0.0, 0.5+size/RADIUS_DIVISOR]`
const RADIUS_DIVISOR: f64 = 16.0;
/// The length is `size/LENGTH_DIVISOR`
const LENGTH_DIVISOR: f32 = 8.0;

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
	to:   (f64, f64, f64)
}

impl Vein {
	pub fn create(size: u32, base: (i32, i32, i32), rng: &mut JavaRng, trig: &TrigLookup) -> Self {
		let size_f32 = size as f32;
		
		let angle = rng.next_f32() * NOTCHIAN_PI;
		let x_size = trig.sin(angle) * size_f32 / LENGTH_DIVISOR;
		let z_size = trig.cos(angle) * size_f32 / LENGTH_DIVISOR;
		
		let from = (
			(base.0       as f32 + x_size) as f64,
			(base.1 + 2 + rng.next_i32(3)) as f64,
			(base.2       as f32 + z_size) as f64
		);
		
		let to = (
			(base.0       as f32 - x_size) as f64,
			(base.1 + 2 + rng.next_i32(3)) as f64,
			(base.2       as f32 - z_size) as f64
		);
		
		Vein { size, size_f64: size as f64, size_f32, from, to }
	}
	
	pub fn blob(&self, index: u32, rng: &mut JavaRng, trig: &TrigLookup) -> Blob {
		let index_f64 = index as f64;
		let index_f32 = index as f32;
		
		let center = (
			lerp_fraction(index_f64, self.size_f64, self.from.0, self.to.0),
			lerp_fraction(index_f64, self.size_f64, self.from.1, self.to.1),
			lerp_fraction(index_f64, self.size_f64, self.from.2, self.to.2)
		);
		
		let radius_multiplier = rng.next_f64() * self.size_f64 / RADIUS_DIVISOR;
		
		let diameter = (trig.sin(index_f32 * NOTCHIAN_PI / self.size_f32) + 1.0f32) as f64 * radius_multiplier + 1.0;
		let radius = diameter / 2.0;
		
		// TODO: i32 casts can overflow.
		let lower = (
			(center.0 - radius).floor() as i32,
			(center.1 - radius).floor() as i32,
			(center.2 - radius).floor() as i32
		);
		
		let upper = (
			(center.0 + radius).floor() as i32,
			(center.1 + radius).floor() as i32,
			(center.2 + radius).floor() as i32
		);
		
		Blob { center, radius, lower, upper }
	}
}

#[derive(Debug)]
pub struct Blob {
	center: (f64, f64, f64),
	radius:  f64,
	lower:  (i32, i32, i32),
	upper:  (i32, i32, i32)
}

/// Preforms linear interpolation using a fraction expressed as `index/size`.
/// Used instead of standard lerp() to preserve operation order.
fn lerp_fraction(index: f64, size: f64, a: f64, b: f64) -> f64 {
	a + (b - a) * index / size
}