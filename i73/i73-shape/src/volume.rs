use java_rand::Random;
use cgmath::Vector3;
use height::Height;
use vocs::position::ColumnPosition;
use i73_noise::octaves::PerlinOctavesVertical;
use i73_base::math;

#[derive(Debug, PartialEq)]
pub struct TriNoiseSettings {
	pub  main_out_scale: f64,
	pub upper_out_scale: f64,
	pub lower_out_scale: f64,
	pub lower_scale:     Vector3<f64>,
	pub upper_scale:     Vector3<f64>,
	pub  main_scale:     Vector3<f64>,
	pub y_size:          usize
}

impl Default for TriNoiseSettings {
	fn default() -> Self {
		TriNoiseSettings {
			 main_out_scale:  20.0,
			upper_out_scale: 512.0,
			lower_out_scale: 512.0,
			lower_scale:     Vector3::new(684.412,        684.412,         684.412       ),
			upper_scale:     Vector3::new(684.412,        684.412,         684.412       ),
			 main_scale:     Vector3::new(684.412 / 80.0, 684.412 / 160.0, 684.412 / 80.0),
			y_size:          17
		}
	}
}

pub struct TriNoiseSource {
	lower:           PerlinOctavesVertical,
	upper:           PerlinOctavesVertical,
	main:            PerlinOctavesVertical,
	 main_out_scale: f64,
	upper_out_scale: f64,
	lower_out_scale: f64
}

impl TriNoiseSource {
	pub fn new(rng: &mut Random, settings: &TriNoiseSettings) -> Self {
		TriNoiseSource {
			lower: PerlinOctavesVertical::new(rng, 16, settings.lower_scale, 0.0, settings.y_size),
			upper: PerlinOctavesVertical::new(rng, 16, settings.upper_scale, 0.0, settings.y_size),
			 main: PerlinOctavesVertical::new(rng,  8, settings. main_scale, 0.0, settings.y_size),
			 main_out_scale: settings. main_out_scale,
			upper_out_scale: settings.upper_out_scale,
			lower_out_scale: settings.lower_out_scale
		}
	}
	
	pub fn sample(&self, point: Vector3<f64>, index: usize) -> f64 {
		let lower = self.lower.generate_override(point, index) / self.lower_out_scale;
		let upper = self.upper.generate_override(point, index) / self.upper_out_scale;
		let main  = self. main.generate_override(point, index) / self. main_out_scale + 0.5;
		
		math::lerp(lower, upper, math::clamp(main, 0.0, 1.0))
	}
}

#[derive(Debug)]
pub struct ShapeSettings {
	/// Stretch value for positions below the height center. Amplifies the effect that distance from
	/// the height center has on the noise value.
	pub seabed_stretch :   f64,
	/// Stretch value for positions above the height center. Amplifies the effect that distance from
	/// the height center has on the noise value.
	pub ground_stretch:    f64,
	/// Controls the distance from the maximum Y value where the tapering function will begin to
	/// have effect. Higher values result in shorter mountains on account of the more aggressive
	/// taper function.
	pub taper_control:     f64,
	/// Stretch value that is applied to all heights. Multiplied with the measured distance from the
	/// height center to influence the reduction of the noise value received from the Tri Noise
	/// generator.
	pub height_stretch:    f64
}

impl ShapeSettings {
	pub fn with_height_stretch(height_stretch: f64) -> Self {
		let mut default = Self::default();
		
		default.height_stretch = height_stretch;
		
		default
	}
}

impl Default for ShapeSettings {
	fn default() -> Self {
		ShapeSettings {
			seabed_stretch:    4.0,
			ground_stretch:    1.0,
			taper_control:     4.0,
			height_stretch:    12.0
		}
	}
}

impl ShapeSettings {
	// TODO: Replace with ShapeSource.
	pub fn compute_noise_value(&self, y: f64, height: Height, tri_noise: f64) -> f64 {
		let distance = y - height.center;

		// Apply different stretch multipliers based on whether the Y value is above or below the
		// height center.
		let distance = distance * if distance < 0.0 { self.seabed_stretch } else { self.ground_stretch };

		let reduction = distance * self.height_stretch / height.chaos;
		let value = tri_noise - reduction;

		// Older generators (ie. inf-20100618) omit this call, resulting in mountains cut off at the
		// height limit. This makes sure that does not happen, by making certain that mountains will
		// taper off well before the limit.
		reduce_upper(value, -10.0, y, self.taper_control, 17.0)
	}
}

pub fn reduce_upper(value: f64, min: f64, y: f64, control: f64, max_y: f64) -> f64 {
	let threshold = max_y - control;
	let divisor   = control - 1.0;
	let factor    = (y.max(threshold) - threshold) / divisor;

	math::lerp_precise(value, min, factor)
}

pub fn reduce_lower(value: f64, min: f64, y: f64, control: f64) -> f64 {
	let divisor   = control - 1.0;
	let factor    = (control - y.min(control)) / divisor;

	math::lerp_precise(value, min, factor)
}

pub fn reduce_cubic(value: f64, distance: f64) -> f64 {
	let factor = 4.0 - distance.min(4.0);
	value - 10.0 * factor.powi(3)
}

pub fn trilinear128(array: &[[[f64; 5]; 17]; 5], position: ColumnPosition) -> f64 {
	debug_assert!(position.y() < 128, "trilinear128 only supports Y values below 128");

	let inner = (
		((position.x() % 4) as f64) / 4.0,
		((position.y() % 8) as f64) / 8.0,
		((position.z() % 4) as f64) / 4.0
	);
	
	let indices = (
		(position.x() / 4) as usize,
		(position.y() / 8) as usize,
		(position.z() / 4) as usize
	);
	
	math::lerp(
		math::lerp(
			math::lerp(
				array[indices.0    ][indices.1    ][indices.2    ],
				array[indices.0    ][indices.1 + 1][indices.2    ],
				inner.1
			),
			math::lerp(
				array[indices.0 + 1][indices.1    ][indices.2    ],
				array[indices.0 + 1][indices.1 + 1][indices.2    ],
				inner.1
			),
			inner.0
		),
		math::lerp(
			math::lerp(
				array[indices.0    ][indices.1    ][indices.2 + 1],
				array[indices.0    ][indices.1 + 1][indices.2 + 1],
				inner.1
			),
			math::lerp(
				array[indices.0 + 1][indices.1    ][indices.2 + 1],
				array[indices.0 + 1][indices.1 + 1][indices.2 + 1],
				inner.1
			),
			inner.0
		),
		inner.2
	)
}