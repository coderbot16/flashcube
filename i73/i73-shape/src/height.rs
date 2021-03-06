use cgmath::{Point2, Vector2, Vector3};
use i73_base::math;
use i73_biome::climate::Climate;
use i73_noise::octaves::PerlinOctaves;
use i73_noise::sample::Sample;
use java_rand::Random;
use vocs::position::LayerPosition;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Height {
	pub center: f64,
	pub chaos: f64,
}

#[derive(Debug, PartialEq)]
pub struct HeightSettings {
	biome_influence_coord_scale: Vector3<f64>,
	biome_influence_scale: f64,
	depth_coord_scale: Vector3<f64>,
	depth_scale: f64,
	depth_base: f64,
}

impl Default for HeightSettings {
	fn default() -> Self {
		HeightSettings {
			biome_influence_coord_scale: Vector3::new(1.121, 0.0, 1.121),
			biome_influence_scale: 512.0,
			depth_coord_scale: Vector3::new(200.0, 0.0, 200.0),
			depth_scale: 8000.0,
			depth_base: 8.5,
		}
	}
}

impl From<HeightSettings81> for HeightSettings {
	fn from(settings: HeightSettings81) -> Self {
		HeightSettings {
			biome_influence_coord_scale: Vector3::new(1.121, 0.0, 1.121),
			biome_influence_scale: 512.0,
			depth_coord_scale: settings.coord_scale,
			depth_scale: settings.out_scale,
			depth_base: settings.base,
		}
	}
}

pub struct HeightSource {
	biome_influence: PerlinOctaves,
	depth: PerlinOctaves,
	biome_influence_scale: f64,
	depth_scale: f64,
	depth_base: f64,
}

impl HeightSource {
	pub fn new(rng: &mut Random, settings: &HeightSettings) -> Self {
		HeightSource {
			biome_influence: PerlinOctaves::new(rng, 10, settings.biome_influence_coord_scale),
			depth: PerlinOctaves::new(rng, 16, settings.depth_coord_scale),
			biome_influence_scale: settings.biome_influence_scale,
			depth_scale: settings.depth_scale,
			depth_base: settings.depth_base,
		}
	}

	pub fn sample(&self, point: Point2<f64>, climate: Climate) -> Height {
		let scaled_noise = self.biome_influence.sample(point) / self.biome_influence_scale;

		// Note: older revisions of the generator do not clamp chaos to 0 (ie. min=Infinity)
		// This can result in chaos becoming negative, producing large "monolith" structures.
		let chaos = math::clamp(climate.influence_factor() * (scaled_noise + 0.5), 0.0, 1.0) + 0.5;

		let mut depth = self.depth.sample(point) / self.depth_scale;

		// Infdev generator excludes this...
		if depth < 0.0 {
			depth *= 0.3
		}

		// Infdev does not place a bound on the depth, and subtracts 3.0 instead of 2.0.
		depth = depth.abs().min(1.0) * 3.0 - 2.0;

		// Infdev uses 1.5 instead of 2.0.
		depth /= if depth < 0.0 { 1.4 } else { 2.0 };

		Height {
			center: self.depth_base + depth * (self.depth_base / 8.0),
			chaos: if depth < 0.0 { 0.5 } else { chaos },
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct HeightSettings81 {
	pub coord_scale: Vector3<f64>,
	pub out_scale: f64,
	pub base: f64,
}

impl HeightSettings81 {
	pub fn with_biome_influence(
		self, biome_influence_coord_scale: Vector3<f64>, biome_influence_scale: f64,
	) -> HeightSettings {
		HeightSettings {
			biome_influence_coord_scale,
			biome_influence_scale,
			depth_coord_scale: self.coord_scale,
			depth_scale: self.out_scale,
			depth_base: self.base,
		}
	}
}

impl Default for HeightSettings81 {
	fn default() -> Self {
		HeightSettings81 {
			coord_scale: Vector3::new(200.0, 0.0, 200.0),
			out_scale: 8000.0,
			base: 8.5,
		}
	}
}

pub struct HeightSource81 {
	depth: PerlinOctaves,
	out_scale: f64,
	base: f64,
}

impl HeightSource81 {
	pub fn new(rng: &mut Random, settings: &HeightSettings81) -> Self {
		HeightSource81 {
			depth: PerlinOctaves::new(rng, 16, settings.coord_scale),
			out_scale: settings.out_scale,
			base: settings.base,
		}
	}

	pub fn sample(&self, point: Point2<f64>, biome_height_center: f64, biome_chaos: f64) -> Height {
		let mut depth = self.depth.sample(point) / self.out_scale;

		if depth < 0.0 {
			depth *= 0.3
		}

		depth = depth.abs().min(1.0) * 3.0 - 2.0;
		depth /= if depth < 0.0 { 1.4 } else { 2.0 };

		depth = depth * 0.2 + biome_height_center;

		Height { center: self.base + depth * (self.base / 8.0), chaos: biome_chaos }
	}
}

/*pub struct BiomeDigestor {
	/// Each cell has a weight assigned. The highest weight is at the center, a max of ~22.36
	weights: [[f32; 5]; 5]
}

impl BiomeDigestor {
	pub fn new() -> Self {
		let mut weights = [[0.0; 5]; 5];

		for x in 0..5 {
			for z in 0..5 {
				// Add 0.2 to prevent a divide by 0, when X/Z are centered.

				let x_relative = (x as i32) - 2;
				let z_relative = (z as i32) - 2;

				let distance_squared = (x_relative*x_relative + z_relative*z_relative) as f32 + 0.2;

				weights[x][z] = 10.0 / ((distance_squared as f64).sqrt() as f32);
			}
		}

		BiomeDigestor { weights }
	}
}*/

/// Converts form lerp coords (5x5) to layer coords (16x16).
/// `
/// 0 => 1
/// 1 => 4
/// 2 => 7
/// 3 => 10
/// 4 => 13
/// `
pub fn lerp_to_layer(lerp: Vector2<u8>) -> LayerPosition {
	LayerPosition::new(lerp.x * 3 + 1, lerp.y * 3 + 1)
}
