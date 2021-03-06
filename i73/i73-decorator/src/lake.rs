use crate::{Decorator, Result};
use i73_base::matcher::BlockMatcher;
use i73_base::block::Block;
use java_rand::Random;
use vocs::component::*;
use vocs::mask::BitCube;
use vocs::position::{CubePosition, ColumnPosition, QuadPosition, LayerPosition};
use vocs::view::QuadMut;

// Since lakes are always 16x8x16, they will never escape the Quad.

#[derive(Debug, Clone)]
pub struct LakeDecorator {
	pub blocks: LakeBlocks,
	pub settings: LakeSettings,
}

impl Decorator for LakeDecorator {
	fn generate(
		&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition,
	) -> Result {
		let mut lower = position.to_centered().unwrap();

		while lower.y() > 0
			&& quad.get(QuadPosition::new(lower.x(), lower.y(), lower.z())) == &self.blocks.carve
		{
			lower = ColumnPosition::new(lower.x(), lower.y() - 1, lower.z());
		}

		// Trying to access blocks below Y=0 returns air, which would cause lake generation to fail.
		if lower.y() < 4 {
			return Ok(());
		}

		lower = ColumnPosition::new(lower.x(), lower.y() - 4, lower.z());

		let mut lake = Lake::new(self.settings.surface);

		lake.fill(LakeSpheroids::new(rng, &self.settings));
		lake.update_border();

		if !self.blocks.check_border(&lake, quad, lower) {
			return Ok(());
		}

		self.blocks.fill_and_carve(&lake, quad, lower);

		Ok(())
	}
}

#[derive(Debug, Clone)]
pub struct LakeBlocks {
	pub is_liquid: BlockMatcher,
	pub is_solid: BlockMatcher,
	pub replaceable: BlockMatcher,
	pub liquid: Block,
	pub carve: Block,
	pub solidify: Option<Block>,
}

impl LakeBlocks {
	pub fn check_border(
		&self, lake: &Lake, quad: &QuadMut<Block>, lower: ColumnPosition,
	) -> bool {
		for x in 0..16 {
			for z in 0..16 {
				for y in 0..lake.surface {
					let at = QuadPosition::new(lower.x() + x, lower.y() + y, lower.z() + z);
					let block = quad.get(at);

					if lake.get(border(x, y, z))
						&& *block != self.liquid && !self.is_solid.matches(block)
					{
						return false;
					}
				}

				for y in lake.surface..8 {
					let at = QuadPosition::new(lower.x() + x, lower.y() + y, lower.z() + z);

					if lake.get(border(x, y, z)) && self.is_liquid.matches(quad.get(at)) {
						return false;
					}
				}
			}
		}

		return true;
	}

	pub fn fill_and_carve(&self, lake: &Lake, quad: &mut QuadMut<Block>, lower: ColumnPosition) {
		quad.ensure_available(self.liquid.clone());
		quad.ensure_available(self.carve.clone());

		let (mut blocks, palette) = quad.freeze_palette();

		let liquid = palette.reverse_lookup(&self.liquid).unwrap();
		let carve = palette.reverse_lookup(&self.carve).unwrap();

		for horizontal_position in LayerPosition::enumerate() {
			let x = horizontal_position.x();
			let z = horizontal_position.z();

			for y in 0..lake.surface {
				let at = QuadPosition::new(lower.x() + x, lower.y() + y, lower.z() + z);

				if lake.get(volume(x, y, z)) {
					blocks.set(at, &liquid);
				}
			}

			for y in lake.surface..8 {
				let at = QuadPosition::new(lower.x() + x, lower.y() + y, lower.z() + z);

				if lake.get(volume(x, y, z)) {
					blocks.set(at, &carve);
				}
			}
		}
	}

	// TODO: grow_grass, solidify_border
}

#[derive(Debug, Clone)]
pub struct LakeSettings {
	pub surface: u8,
	pub min_spheroids: u32,
	pub add_spheroids: u32,
}

impl Default for LakeSettings {
	fn default() -> Self {
		LakeSettings { surface: 4, min_spheroids: 4, add_spheroids: 3 }
	}
}

pub struct LakeSpheroids<'r> {
	remaining_spheroids: u32,
	rng: &'r mut Random,
}

impl<'r> LakeSpheroids<'r> {
	pub fn new(rng: &'r mut Random, settings: &LakeSettings) -> Self {
		let remaining_spheroids =
			settings.min_spheroids + rng.next_u32_bound(settings.add_spheroids + 1);

		LakeSpheroids { remaining_spheroids, rng }
	}
}

impl<'r> Iterator for LakeSpheroids<'r> {
	type Item = Spheroid;

	fn next(&mut self) -> Option<Self::Item> {
		if self.remaining_spheroids <= 0 {
			return None;
		}

		self.remaining_spheroids -= 1;

		let diameter = (
			self.rng.next_f64() * 6.0 + 3.0,
			self.rng.next_f64() * 4.0 + 2.0,
			self.rng.next_f64() * 6.0 + 3.0,
		);

		let radius = (diameter.0 / 2.0, diameter.1 / 2.0, diameter.2 / 2.0);

		let center = (
			self.rng.next_f64() * (16.0 - diameter.0 - 2.0) + 1.0 + radius.0,
			self.rng.next_f64() * (8.0 - diameter.1 - 4.0) + 2.0 + radius.1,
			self.rng.next_f64() * (16.0 - diameter.2 - 2.0) + 1.0 + radius.2,
		);

		Some(Spheroid { radius, center })
	}
}

#[derive(Debug)]
pub struct Spheroid {
	pub center: (f64, f64, f64),
	pub radius: (f64, f64, f64),
}

pub fn volume(x: u8, y: u8, z: u8) -> CubePosition {
	CubePosition::new(x, y % 8, z)
}

pub fn border(x: u8, y: u8, z: u8) -> CubePosition {
	CubePosition::new(x, (y % 8) + 8, z)
}

/// Uses a BitCube to store both the volume and the border blocks.
/// Lakes are 16x8x16. A BitCube is 16x16x16.
/// For compactness, these two masks are stacked on top of each other.
pub struct Lake {
	shape: BitCube,
	surface: u8,
}

impl Lake {
	pub fn new(surface: u8) -> Self {
		Lake { shape: BitCube::default(), surface }
	}

	pub fn clear(&mut self) {
		self.shape.fill(false)
	}

	pub fn set_or(&mut self, at: CubePosition, value: bool) {
		use vocs::mask::Mask;
		self.shape.set_or(at, value)
	}

	pub fn set(&mut self, at: CubePosition, value: bool) {
		self.shape.set(at, value)
	}

	pub fn get(&self, at: CubePosition) -> bool {
		self.shape[at]
	}

	pub fn fill(&mut self, spheroids: LakeSpheroids) {
		for spheroid in spheroids {
			self.add_spheroid(spheroid);
		}
	}

	pub fn add_spheroid(&mut self, spheroid: Spheroid) {
		// TODO: Reduce size of possible bounding box.
		for x in 1..15 {
			for y in 1..7 {
				for z in 1..15 {
					let axis_distances = (
						(x as f64 - spheroid.center.0) / spheroid.radius.0,
						(y as f64 - spheroid.center.1) / spheroid.radius.1,
						(z as f64 - spheroid.center.2) / spheroid.radius.2,
					);

					let distance_squared = axis_distances.0 * axis_distances.0
						+ axis_distances.1 * axis_distances.1
						+ axis_distances.2 * axis_distances.2;

					self.set_or(volume(x, y, z), distance_squared < 1.0);
				}
			}
		}
	}

	pub fn update_border(&mut self) {
		// Main volume
		for x in 1..15 {
			for y in 1..7 {
				for z in 1..15 {
					let is_border =
						!self.get(volume(x, y, z))
							&& (self.get(volume(x + 1, y, z))
								|| self.get(volume(x - 1, y, z)) || self.get(volume(x, y + 1, z))
								|| self.get(volume(x, y - 1, z)) || self.get(volume(x, y, z + 1))
								|| self.get(volume(x, y, z - 1)));

					self.set(border(x, y, z), is_border);
				}
			}
		}

		// Top and bottom face
		for x in 1..15 {
			for z in 1..15 {
				let bottom = self.get(volume(x, 1, z));
				let top = self.get(volume(x, 7 - 1, z));

				self.set(border(x, 0, z), bottom);
				self.set(border(x, 7, z), top);
			}
		}

		// Z=0 / Z=Max faces
		for x in 1..15 {
			for y in 1..7 {
				let min = self.get(volume(x, y, 1));
				let max = self.get(volume(x, y, 15 - 1));

				self.set(border(x, y, 0), min);
				self.set(border(x, y, 15), max);
			}
		}

		// X=0 / X=Max faces
		for z in 1..15 {
			for y in 1..7 {
				let min = self.get(volume(1, y, z));
				let max = self.get(volume(15 - 1, y, z));

				self.set(border(0, y, z), min);
				self.set(border(15, y, z), max);
			}
		}

		// Skip the edge/corner cases (literally) as they cannot possibly fulfill any of the criteria.
		// TODO: Not clearing these may lead to corruption.
	}
}
