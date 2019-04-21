use java_rand::Random;
use cgmath::Vector3;
use i73_noise::octaves::PerlinOctaves;
use i73_biome::climate::{ClimateSettings, ClimateSource};
use i73_biome::Lookup;
use i73_shape::height::{HeightSettings, HeightSource};
use i73_shape::volume::{TriNoiseSettings, TriNoiseSource, ShapeSettings};

use overworld::shape::{ShapeBlocks, ShapePass};
use overworld::paint::{PaintBlocks, PaintPass};

pub struct Settings {
	pub shape_blocks: ShapeBlocks,
	pub paint_blocks: PaintBlocks,
	pub tri:          TriNoiseSettings,
	pub height:       HeightSettings,
	pub field:        ShapeSettings,
	pub sea_coord:    u8,
	pub beach:        Option<(u8, u8)>,
	pub max_bedrock_height: Option<u8>,
	pub climate:      ClimateSettings
}

impl Default for Settings {
	fn default() -> Self {
		Settings {
			shape_blocks: ShapeBlocks::default(),
			paint_blocks: PaintBlocks::default(),
			tri:          TriNoiseSettings::default(),
			height:       HeightSettings::default(),
			field:        ShapeSettings::default(),
			sea_coord:    63,
			beach:        Some((59, 65)),
			max_bedrock_height: Some(5),
			climate:      ClimateSettings::default()
		}
	}
}

pub fn passes(seed: u64, settings: Settings, biome_lookup: Lookup) -> (ClimateSource, ShapePass, PaintPass) {
	let mut rng = Random::new(seed);

	let tri = TriNoiseSource::new(&mut rng, &settings.tri);

	// TODO: The PerlinOctaves implementation currently does not support noise on arbitrary Y coordinates.
	// Oddly, this "feature" is what causes the sharp walls in beach/biome surfaces.
	// It is a mystery why the feature exists in the first place.

	let sand      = PerlinOctaves::new(&mut rng.clone(), 4, Vector3::new(1.0 / 32.0, 1.0 / 32.0,        1.0)); // Vertical,   Z =   0.0
	let gravel    = PerlinOctaves::new(&mut rng,                        4, Vector3::new(1.0 / 32.0,        1.0, 1.0 / 32.0)); // Horizontal
	let thickness = PerlinOctaves::new(&mut rng,                        4, Vector3::new(1.0 / 16.0, 1.0 / 16.0, 1.0 / 16.0)); // Vertical,   Z =   0.0

	let height  = HeightSource::new(&mut rng, &settings.height);
	let field   = settings.field;

	(
		ClimateSource::new(seed, settings.climate),
		ShapePass {
			blocks: settings.shape_blocks,
			tri,
			height,
			field
		},
		PaintPass {
			lookup: biome_lookup,
			blocks: settings.paint_blocks,
			sand,
			gravel,
			thickness,
			sea_coord: settings.sea_coord,
			beach: settings.beach,
			max_bedrock_height: settings.max_bedrock_height
		}
	)
}