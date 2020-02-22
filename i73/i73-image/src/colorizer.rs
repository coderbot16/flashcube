use i73_base::math;
use i73_biome::climate::Climate;
use image::Rgb;

// Foliage.png stuff...
/*const RAINFOREST: Rgb<u8> = Rgb { data: [0x1A, 0xBF, 0x00] };
const DESERT: Rgb<u8> = Rgb { data: [0xAE, 0xA4, 0x2A] };
const COLDEST: Rgb<u8> = Rgb { data: [0x60, 0xA1, 0x7B] };*/

pub const RAINFOREST: Rgb<u8> = Rgb { data: [0x47, 0xCD, 0x33] };
pub const DESERT: Rgb<u8> = Rgb { data: [0xBF, 0xB7, 0x55] };
pub const COLDEST: Rgb<u8> = Rgb { data: [0x80, 0xB4, 0x97] };

pub fn coordinates_to_climate(x: u32, y: u32) -> Climate {
	Climate::new(1.0 - (x as f64) / 255.0, 1.0 - (y as f64) / 255.0)
}

pub fn color_coordinates(climate: Climate) -> (u32, u32) {
	let x = (1.0 - climate.temperature()) * 255.0;
	let y = (1.0 - climate.adjusted_rainfall()) * 255.0;

	(x as u32, y as u32)
}

pub fn colorize_grass(climate: Climate) -> Rgb<u8> {
	let adjusted_rainfall = climate.adjusted_rainfall();
	let temperature = climate.temperature();

	fn lerp_color(a: u8, b: u8, t: f64) -> f64 {
		math::lerp(a as f64, b as f64, t)
	}

	fn lerp_color_final(a: f64, b: u8, t: f64) -> u8 {
		math::lerp(a, b as f64, t) as u8
	}

	Rgb {
		data: [
			lerp_color_final(
				lerp_color(COLDEST.data[0], DESERT.data[0], temperature),
				RAINFOREST.data[0],
				adjusted_rainfall,
			),
			lerp_color_final(
				lerp_color(COLDEST.data[1], DESERT.data[1], temperature),
				RAINFOREST.data[1],
				adjusted_rainfall,
			),
			lerp_color_final(
				lerp_color(COLDEST.data[2], DESERT.data[2], temperature),
				RAINFOREST.data[2],
				adjusted_rainfall,
			),
		],
	}
}
