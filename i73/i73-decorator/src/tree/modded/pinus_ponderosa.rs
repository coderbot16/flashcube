// Based on https://github.com/Team-RTG/Realistic-Terrain-Generation/blob/1.12.2-dev/src/main/java/rtg/api/world/gen/feature/tree/rtg/TreeRTGPinusPonderosa.java
// Licensed under the GPLv3.

use crate::tree::{FoliageLayer, TreeBlocks};
use crate::{Decorator, Result};
use i73_base::block::Block;
use java_rand::Random;
use vocs::position::{dir, Dir, Offset, QuadPosition};
use vocs::view::QuadMut;

#[derive(Default)]
pub struct PinusPonderosaTreeDecorator {
	blocks: TreeBlocks
}

impl Decorator for PinusPonderosaTreeDecorator {
	fn generate(
		&self, quad: &mut QuadMut<Block>, rng: &mut Random, position: QuadPosition,
	) -> Result {
		/*let tree = self.settings.tree(rng, position.y());

		if tree.leaves_max_y > 128 {
			return Ok(());
		}*/

		let below = match position.offset(dir::Down) {
			Some(below) => below,
			None => return Ok(()),
		};

		if !self.blocks.soil.matches(quad.get(below)) {
			return Ok(());
		}

		// TODO: Better soil check.

		// TODO: Check bounding box

		quad.set_immediate(below, &self.blocks.new_soil);
		quad.set_immediate(below.offset(dir::PlusX).unwrap(), &self.blocks.new_soil);
		quad.set_immediate(below.offset(dir::MinusX).unwrap(), &self.blocks.new_soil);
		quad.set_immediate(below.offset(dir::PlusZ).unwrap(), &self.blocks.new_soil);
		quad.set_immediate(below.offset(dir::MinusZ).unwrap(), &self.blocks.new_soil);

		quad.ensure_available(self.blocks.log.clone());
		quad.ensure_available(self.blocks.foliage.clone());

		let (mut blocks, palette) = quad.freeze_palette();

		let log = palette.reverse_lookup(&self.blocks.log).unwrap();
		let foliage = palette.reverse_lookup(&self.blocks.foliage).unwrap();

		let trunkSize = (rng.next_u32_bound(15) + 20) as u8;
		let wideTrunkSize = (trunkSize + 7) / 8;

		// Place wide log
		for direction in [(-1i8, 0i8, 0i8), (1, 0, 0), (0, 0, 1), (0, 0, -1)] {
			let height: u8 = (wideTrunkSize as u32 + rng.next_u32_bound(wideTrunkSize as u32 * 2)) as u8;
			let offset = position.offset(direction).unwrap();

			for dZ in 0..height {
				let position = QuadPosition::new(offset.x(), offset.y() + dZ, offset.z());
				blocks.set(position, &log);
			}
		}


		let mut pX = 0;
		let mut pZ = 0;

		for y in (position.y() + 5)..(position.y() + trunkSize) {
			let diff = (trunkSize - (y - position.y())) as u32;

			let chance = if (diff < 7) {
				1
			} else if (diff < 12) {
				2
			} else {
				7
			};

			if (rng.next_u32_bound(chance) == 0) {
				let mut dX = -1 + (rng.next_u32_bound(3) as i8);
				let mut dZ = -1 + (rng.next_u32_bound(3) as i8);

				if dX == 0 && dZ == 0 {
					dX = -1 + (rng.next_u32_bound(3) as i8);
					dZ = -1 + (rng.next_u32_bound(3) as i8);
				}

				if pX == dX && rng.next_bool() {
					dX = -dX;
				}

				if pZ == dZ && rng.next_bool() {
					dZ = -dZ;
				}

				pX = dX;
				pZ = dZ;

				let fposition = QuadPosition::new(position.x(), y as u8, position.z()).offset((dX, 0, dZ)).unwrap();
				let layer = FoliageLayer { position: fposition, radius: 1 };
	
				layer.place(&mut blocks, &foliage, &palette, &self.blocks.replace);
				layer.place_corners(&mut blocks, &foliage, &palette, &self.blocks.replace, |y| true);

				blocks.set(fposition, &log);

				let fposition = QuadPosition::new(position.x(), (y as u8) + 1, position.z()).offset((dX, 0, dZ)).unwrap();
				let layer = FoliageLayer { position: fposition, radius: 1 };
	
				layer.place(&mut blocks, &foliage, &palette, &self.blocks.replace);
			}
		}

		for y in position.y()..(position.y() + trunkSize) {
			let position = QuadPosition::new(position.x(), y, position.z());

			if self.blocks.replace.matches(blocks.get(position, &palette)) {
				blocks.set(position, &log);
			}
		}

		Ok(())
	}
}

