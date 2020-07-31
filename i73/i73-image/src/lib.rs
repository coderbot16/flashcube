extern crate cgmath;
extern crate frontend;
extern crate i73_base;
extern crate i73_biome;
extern crate i73_noise;
extern crate i73_shape;
extern crate i73_terrain;
extern crate image;
extern crate vocs;

pub mod colorizer;
pub mod renderer;
pub mod stitcher;

pub struct Rgb {
	pub red: u8,
	pub green: u8,
	pub blue: u8
}

impl Rgb {
	pub fn gray(gray: u8) -> Self {
		Rgb {
			red: gray,
			green: gray,
			blue: gray
		}
	}
}

impl Into<image::Rgb<u8>> for Rgb {
	fn into(self) -> image::Rgb<u8> {
		image::Rgb([self.red, self.green, self.blue])
	}
}
