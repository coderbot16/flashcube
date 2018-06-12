use position::{ChunkPosition, LayerPosition, Offset};
use mask::{Mask, LayerMask};
use view::Directional;
use std::ops::IndexMut;
use component::{Component, ChunkStorage, LayerStorage};

pub type SpillChunkMask = SpillChunk<bool>;

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

impl<D, C> StorageOffset<ChunkPosition, D, C> for SpillChunk<C>
	where Directional<C::Layer>: IndexMut<D, Output=C::Layer>,
		  ChunkPosition: Offset<D, Spill=LayerPosition>,
		  D: Copy,
		  C: Component {
	fn set_offset(&mut self, position: ChunkPosition, d: D, value: C) {
		match position.offset_spilling(d) {
			Ok(position) => self.primary.set(position, value),
			Err(layer) => self.spills[d].set(layer, value)
		}
	}
}

impl<D> MaskOffset<ChunkPosition, D> for SpillChunk<bool>
	where Directional<LayerMask>: IndexMut<D, Output=LayerMask>,
		  ChunkPosition: Offset<D, Spill=LayerPosition>,
		  D: Copy {
	fn set_offset_true(&mut self, position: ChunkPosition, d: D) {
		match position.offset_spilling(d) {
			Ok(position) => self.primary.set_true(position),
			Err(layer) => self.spills[d].set_true(layer)
		}
	}

	fn set_offset_false(&mut self, position: ChunkPosition, d: D) {
		match position.offset_spilling(d) {
			Ok(position) => self.primary.set_true(position),
			Err(layer) => self.spills[d].set_false(layer)
		}
	}
}