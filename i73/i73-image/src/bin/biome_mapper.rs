extern crate cgmath;
extern crate frontend;
extern crate i73_base;
extern crate i73_biome;
extern crate i73_image;
extern crate i73_noise;
extern crate i73_shape;
extern crate i73_terrain;
extern crate image;

extern crate java_rand;
extern crate vocs;

use image::{Rgb, RgbImage};
use std::fs;

use cgmath::{Point2, Vector3};
use i73_biome::climate::{ClimateSettings, ClimateSource};
use i73_noise::octaves::PerlinOctaves;
use i73_noise::sample::Sample;
use i73_shape::height::{HeightSettings, HeightSource};
use i73_shape::volume::TriNoiseSettings;

fn main() {
	// Initialization
	let seed = 8399452073110208023;

	let mut rng = java_rand::Random::new(seed);

	let _tri = i73_shape::volume::TriNoiseSource::new(&mut rng, &TriNoiseSettings::default());

	let _sand = PerlinOctaves::new(&mut rng.clone(), 4, Vector3::new(1.0 / 32.0, 1.0 / 32.0, 1.0)); // Vertical,   Z =   0.0
	let _gravel = PerlinOctaves::new(&mut rng, 4, Vector3::new(1.0 / 32.0, 1.0, 1.0 / 32.0)); // Horizontal
	let _thickness =
		PerlinOctaves::new(&mut rng, 4, Vector3::new(1.0 / 16.0, 1.0 / 16.0, 1.0 / 16.0)); // Vertical,   Z =   0.0

	let height_source = HeightSource::new(&mut rng, &HeightSettings::default());
	let climates = ClimateSource::new(seed, ClimateSettings::default());

	// Image generation

	generate_image("heightmap", (2048, 2048), 4, |x, z| {
		let point = Point2 { x: x as f64, y: z as f64 };

		let climate = climates.sample(point * 4.0);
		let height = height_source.sample(point, climate);

		let scaled_center = ((height.center / 17.0) * 255.0) as u8;
		// let scaled_chaos = ((height.chaos - 0.5) * 255.0) as u8;

		Rgb { data: [scaled_center, scaled_center, scaled_center] }

		/*Rgb {
			data: [
				scaled_chaos,
				scaled_chaos,
				scaled_chaos
			]
		}*/
	});
}

fn generate_image<F>(name: &str, size: (u32, u32), scale: u32, f: F)
where
	F: Fn(u32, u32) -> Rgb<u8>,
{
	let mut map = RgbImage::new(size.0, size.1);

	for z in 0..size.1 {
		if (z % 256) == 0 {
			println!("{:.2}%", ((z as f64) / (size.1 as f64)) * 100.0);
		}

		for x in 0..size.0 {
			map.put_pixel(x, z, f(x, z));
		}
	}

	println!("Generation complete, saving image...");
	fs::create_dir_all("out/image/").unwrap();

	if scale <= 1 {
		map.save(format!("out/image/{}.png", name)).unwrap();
	} else {
		let resized = image::imageops::resize(
			&map,
			size.0 * scale,
			size.1 * scale,
			image::FilterType::Triangle,
		);
		resized.save(format!("out/image/{}.png", name)).unwrap();
	}
}
