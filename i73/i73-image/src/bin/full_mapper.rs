extern crate image;
extern crate i73_base;
extern crate i73_image;
extern crate i73_biome;
extern crate i73_noise;
extern crate i73_shape;
extern crate i73_terrain;
extern crate frontend;
extern crate cgmath;

extern crate java_rand;
extern crate vocs;

use i73_image::stitcher;
use i73_image::renderer::full::create_renderer;

fn main() {
	// Farlands
	// generate_full_image("world", (4, 4), (784400, 0));

	stitcher::generate_stitched_image(|| { create_renderer(8399452073110208023) }, "world", (8, 8), (0, 0));
}

