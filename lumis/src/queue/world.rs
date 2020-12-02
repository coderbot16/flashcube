use crate::queue::SectorSpills;

use std::collections::HashMap;

use vocs::mask::BitCube;
use vocs::mask::BitLayer;
use vocs::position::{CubePosition, GlobalSectorPosition, LayerPosition};
use vocs::unpacked::Layer;
use vocs::world::sector::Sector;

#[derive(Copy, Clone)]
enum Phase {
	Odd,
	Even,
}

impl Phase {
	fn from_position(position: GlobalSectorPosition) -> Self {
		let is_odd = position.x().wrapping_add(position.z()) & 1 == 1;

		if is_odd {
			Phase::Odd
		} else {
			Phase::Even
		}
	}

	fn next(self) -> Self {
		match self {
			Phase::Odd => Phase::Even,
			Phase::Even => Phase::Odd,
		}
	}
}

pub struct WorldQueue {
	odd: HashMap<GlobalSectorPosition, Sector<BitCube>>,
	even: HashMap<GlobalSectorPosition, Sector<BitCube>>,
	phase: Phase,
}

impl WorldQueue {
	pub fn new() -> WorldQueue {
		WorldQueue { odd: HashMap::new(), even: HashMap::new(), phase: Phase::Odd }
	}

	pub fn enqueue_spills(&mut self, position: GlobalSectorPosition, spills: SectorSpills) {
		let spills = spills.0.split();

		self.spill(
			GlobalSectorPosition::new(position.x() + 1, position.z()),
			spills.plus_x,
			|layer_position| CubePosition::new(0, layer_position.x(), layer_position.z()),
			|mask, layer| mask.layer_zy_mut(0).combine(&layer),
		);

		self.spill(
			GlobalSectorPosition::new(position.x() - 1, position.z()),
			spills.minus_x,
			|layer_position| CubePosition::new(15, layer_position.x(), layer_position.z()),
			|mask, layer| mask.layer_zy_mut(15).combine(&layer),
		);

		self.spill(
			GlobalSectorPosition::new(position.x(), position.z() + 1),
			spills.plus_z,
			|layer_position| CubePosition::new(layer_position.x(), layer_position.z(), 0),
			|mask, layer| mask.layer_yx_mut(0).combine(&layer),
		);

		self.spill(
			GlobalSectorPosition::new(position.x(), position.z() - 1),
			spills.minus_z,
			|layer_position| CubePosition::new(layer_position.x(), layer_position.z(), 15),
			|mask, layer| mask.layer_yx_mut(15).combine(&layer),
		);
	}

	fn sector_masks(&mut self, position: GlobalSectorPosition) -> &mut Sector<BitCube> {
		match Phase::from_position(position) {
			Phase::Odd => &mut self.odd,
			Phase::Even => &mut self.even,
		}
		.entry(position)
		.or_insert_with(Sector::new)
	}

	fn spill<P, M>(
		&mut self, origin: GlobalSectorPosition, layer: Layer<Option<BitLayer>>, position: P,
		mut merge: M,
	) where
		P: Fn(LayerPosition) -> CubePosition,
		M: FnMut(&mut BitCube, BitLayer),
	{
		use vocs::component::LayerStorage;

		for (index, spilled) in layer.into_inner().into_vec().drain(..).enumerate() {
			let spilled: Option<BitLayer> = spilled;

			let spilled = match spilled {
				Some(mask) => mask,
				None => continue,
			};

			if spilled.is_filled(false) {
				continue;
			}

			let layer_position = LayerPosition::from_zx(index as u8);
			let chunk_position = position(layer_position);

			// TODO: Don't repeatedly perform hashmap lookups
			let sector = self.sector_masks(origin);

			merge(sector.get_or_create_mut(chunk_position), spilled);
		}
	}

	pub fn flip(&mut self) -> Option<HashMap<GlobalSectorPosition, Sector<BitCube>>> {
		match (self.even.is_empty(), self.odd.is_empty()) {
			(true, true) => {
				self.phase = Phase::Odd;

				None
			}
			(false, true) => {
				self.phase = Phase::Even;

				Some(std::mem::replace(&mut self.even, HashMap::new()))
			}
			(true, false) => {
				self.phase = Phase::Odd;

				Some(std::mem::replace(&mut self.odd, HashMap::new()))
			}
			(false, false) => {
				self.phase = self.phase.next();

				Some(std::mem::replace(
					match self.phase {
						Phase::Odd => &mut self.odd,
						Phase::Even => &mut self.even,
					},
					HashMap::new(),
				))
			}
		}
	}
}
