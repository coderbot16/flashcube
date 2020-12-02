use crate::position::{CubePosition, LayerPosition, Offset};
use crate::mask::{Mask, LayerMask};
use crate::view::Directional;
use std::ops::IndexMut;
use crate::component::{Component, ChunkStorage, LayerStorage};

pub type SpillBitCube = SpillChunk<bool>;

pub trait MaskOffset<P, D> {
	fn set_offset_true(&mut self, position: P, offset: D);
	fn set_offset_false(&mut self, position: P, offset: D);
}

pub trait StorageOffset<P, D, C: Component> {
	fn set_offset(&mut self, position: P, offset: D, value: C);
}

#[derive(Default, Clone)]
pub struct SpillChunk<C: Component> {
	pub primary: C::Chunk,
	pub spills: Directional<C::Layer>
}

impl<D, C> StorageOffset<CubePosition, D, C> for SpillChunk<C>
	where Directional<C::Layer>: IndexMut<D, Output=C::Layer>,
		  CubePosition: Offset<D, Spill=LayerPosition>,
		  D: Copy,
		  C: Component {
	fn set_offset(&mut self, position: CubePosition, d: D, value: C) {
		match position.offset_spilling(d) {
			Ok(position) => self.primary.set(position, value),
			Err(layer) => self.spills[d].set(layer, value)
		}
	}
}

impl<D> MaskOffset<CubePosition, D> for SpillChunk<bool>
	where Directional<LayerMask>: IndexMut<D, Output=LayerMask>,
		  CubePosition: Offset<D, Spill=LayerPosition>,
		  D: Copy {
	fn set_offset_true(&mut self, position: CubePosition, d: D) {
		match position.offset_spilling(d) {
			Ok(position) => self.primary.set_true(position),
			Err(layer) => self.spills[d].set_true(layer)
		}
	}

	fn set_offset_false(&mut self, position: CubePosition, d: D) {
		match position.offset_spilling(d) {
			Ok(position) => self.primary.set_true(position),
			Err(layer) => self.spills[d].set_false(layer)
		}
	}
}