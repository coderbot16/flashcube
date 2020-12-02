extern crate i73_base;
extern crate i73_biome;
extern crate i73_image;
extern crate image;

use image::{Rgb, RgbImage};
use std::io::BufReader;
use std::{fs, i32};

use i73_image::colorizer::{colorize_grass, coordinates_to_climate};

fn main() {
	let reference_file = fs::File::open("out/image/grasscolor.png").unwrap();
	let reference =
		image::load(BufReader::new(reference_file), image::ImageFormat::Png).unwrap().to_rgb();
	let mut map = RgbImage::new(256, 256);
	let mut diff = RgbImage::new(256, 256);

	/*for z in 0..=4096u32 {
		for x in 0..=4096u32 {
			let temperature = (x as f64) / 4096.0;
			let rainfall = (z as f64) / 4096.0;

			let (px, py) = color_coordinates(temperature, rainfall);
			let color = colorize(temperature, rainfall);

			map.put_pixel(px, py, color);
		}
	}*/

	for x in 0..256 {
		for y in x..256 {
			let climate = coordinates_to_climate(x, y);
			let color = colorize_grass(climate);

			map.put_pixel(x, y, color.into());
		}

		for y in 0..x {
			map.put_pixel(x, y, Rgb ([255, 255, 255]));
		}
	}

	for y in 0..256 {
		for x in 0..256 {
			let ref_pixel = *reference.get_pixel(x, y);
			let map_pixel = *map.get_pixel(x, y);

			diff.put_pixel(
				x,
				y,
				Rgb ([
					i32::abs(ref_pixel.0[0] as i32 - map_pixel.0[0] as i32) as u8,
					i32::abs(ref_pixel.0[1] as i32 - map_pixel.0[1] as i32) as u8,
					i32::abs(ref_pixel.0[2] as i32 - map_pixel.0[2] as i32) as u8,
				]),
			);
		}
	}

	fs::create_dir_all("out/image/").unwrap();
	map.save("out/image/grass_created.png").unwrap();
	diff.save("out/image/diff.png").unwrap();
}
