use crate::queue::ChunkSpills;
use std::mem;
use vocs::component::LayerStorage;
use vocs::mask::ChunkMask;
use vocs::mask::LayerMask;
use vocs::position::{dir, CubePosition, LayerPosition, Offset};
use vocs::unpacked::Layer;
use vocs::view::{Directional, SplitDirectional};
use vocs::world::sector::Sector;

pub struct SectorQueue {
	/// The queue currently being emptied.
	front: Sector<ChunkMask>,
	/// The queue currently being filled.
	back: Sector<ChunkMask>,
	spills: SectorSpills,
}

impl SectorQueue {
	pub fn new() -> Self {
		SectorQueue { front: Sector::new(), back: Sector::new(), spills: SectorSpills::default() }
	}

	pub fn reset_from_mask(&mut self, mask: Sector<ChunkMask>) {
		// TODO: Properly clear spills & front mask
		assert!(self.empty());
		self.reset_spills();

		self.back = mask;
	}

	pub fn empty(&self) -> bool {
		self.front.is_empty()
	}

	pub fn reset_spills(&mut self) -> SectorSpills {
		assert!(self.front.is_empty());

		std::mem::replace(&mut self.spills, SectorSpills::default())
	}

	pub fn pop_first(&mut self) -> Option<(CubePosition, ChunkMask)> {
		self.front.pop_first()
	}

	pub fn flip(&mut self) -> bool {
		mem::swap(&mut self.front, &mut self.back);

		!self.front.is_empty()
	}

	pub fn enqueue_spills(&mut self, origin: CubePosition, spills: ChunkSpills) {
		let spills = spills.split();

		self.spill(origin, dir::Up, spills.up, |mask, layer| mask.layer_zx_mut(0).combine(&layer));
		self.spill(origin, dir::Down, spills.down, |mask, layer| {
			mask.layer_zx_mut(15).combine(&layer)
		});
		self.spill(origin, dir::PlusX, spills.plus_x, |mask, layer| {
			mask.layer_zy_mut(0).combine(&layer)
		});
		self.spill(origin, dir::MinusX, spills.minus_x, |mask, layer| {
			mask.layer_zy_mut(15).combine(&layer)
		});
		self.spill(origin, dir::PlusZ, spills.plus_z, |mask, layer| {
			mask.layer_yx_mut(0).combine(&layer)
		});
		self.spill(origin, dir::MinusZ, spills.minus_z, |mask, layer| {
			mask.layer_yx_mut(15).combine(&layer)
		});
	}

	fn spill<D, F>(&mut self, origin: CubePosition, dir: D, layer: LayerMask, mut f: F)
	where
		CubePosition: Offset<D, Spill = LayerPosition>,
		F: FnMut(&mut ChunkMask, LayerMask),
		D: Copy,
		Directional<Layer<Option<LayerMask>>>:
			std::ops::IndexMut<D, Output = Layer<Option<LayerMask>>>,
	{
		// If the layer is empty, don't bother adding / merging it.
		if layer.is_filled(false) {
			return;
		}

		// Either merge it with a local chunk mask, or add it to the neighboring spills.
		match origin.offset_spilling(dir) {
			Ok(position) => f(self.back.get_or_create_mut(position), layer),
			Err(spilled) => {
				let slot = &mut self.spills.0[dir][spilled];

				match slot.as_mut() {
					Some(existing) => *existing |= &layer,
					None => *slot = Some(layer),
				}
			}
		}
	}
}

pub struct SectorSpills(/*TODO: make private*/ pub Directional<Layer<Option<LayerMask>>>);

impl Default for SectorSpills {
	fn default() -> SectorSpills {
		SectorSpills(Directional::combine(SplitDirectional {
			plus_x: Layer::default(),
			minus_x: Layer::default(),
			up: Layer::default(),
			down: Layer::default(),
			plus_z: Layer::default(),
			minus_z: Layer::default(),
		}))
	}
}
