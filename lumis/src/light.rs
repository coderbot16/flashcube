use bit_vec::BitVec;
use crate::heightmap::ChunkHeightMap;
use crate::queue::Queue;
use vocs::mask::{Mask, LayerMask};
use vocs::nibbles::{u4, ChunkNibbles, LayerNibbles, BulkNibbles};
use vocs::packed::ChunkPacked;
use vocs::component::*;
use vocs::position::{ChunkPosition, LayerPosition, Offset, dir};
use vocs::view::{SpillChunkMask, MaskOffset, Directional};
use std::cmp::max;

#[derive(Debug)]
pub struct Lighting<'n, S> where S: LightSources {
	data: &'n mut ChunkNibbles,
	neighbors: Directional<&'n ChunkNibbles>,
	sources: S,
	opacity: BulkNibbles
}

impl<'n, S> Lighting<'n, S> where S: LightSources {
	pub fn new(data: &'n mut ChunkNibbles, neighbors: Directional<&'n ChunkNibbles>, sources: S, opacity: BulkNibbles) -> Self {
		Lighting {
			data,
			neighbors,
			sources,
			opacity
		}
	}
	
	pub fn set(&mut self, queue: &mut Queue, at: ChunkPosition, light: u4) {
		if light != self.get(at) {
			self.data.set(at, light);
			queue.enqueue_neighbors(at);
		}
	}
	
	pub fn get(&self, at: ChunkPosition) -> u4 {
		self.data.get(at)
	}
	
	pub fn initial(&mut self, chunk: &ChunkPacked, queue: &mut Queue) {
		self.sources.initial(chunk, &mut self.data, queue.mask_mut())
	}
	
	pub fn step(&mut self, chunk: &ChunkPacked, queue: &mut Queue) -> bool {
		if !queue.flip() {
			return false;
		}

		while let Some(at) = queue.next() {
			let max_value = max(
				max(
					max(
						at.offset(dir::MinusX).map(|at| self.get(at)).unwrap_or(self.neighbors[dir::MinusX].get(at.with_x(15))),
						at.offset(dir::PlusX ).map(|at| self.get(at)).unwrap_or(self.neighbors[dir::PlusX].get(at.with_x(0)))
					),
					max(
						at.offset(dir::MinusZ).map(|at| self.get(at)).unwrap_or(self.neighbors[dir::MinusZ].get(at.with_z(15))),
						at.offset(dir::PlusZ ).map(|at| self.get(at)).unwrap_or(self.neighbors[dir::PlusZ].get(at.with_z(0)))
					)
				),
				max(
					at.offset(dir::Down).map(|at| self.get(at)).unwrap_or(self.neighbors[dir::Down].get(at.with_y(15))),
					at.offset(dir::Up  ).map(|at| self.get(at)).unwrap_or(self.neighbors[dir::Up].get(at.with_y(0)))
				)
			);

			let opacity = self.opacity.get(chunk.get(at) as usize);

			let new_light = max (
				max_value.saturating_sub(u4::new(1)),
				self.sources.emission(chunk, at)
			).saturating_sub(opacity);

			self.set(queue, at, new_light);
		}

		return true;
	}
	
	pub fn finish(&mut self, chunk: &ChunkPacked, queue: &mut Queue) {
		while self.step(chunk, queue) {}
	}
	
	pub fn decompose(self) -> (&'n mut ChunkNibbles, S) {
		(self.data, self.sources)
	}
	
	pub fn opacity(&self) -> &BulkNibbles {
		&self.opacity
	}
}

pub trait LightSources {
	fn emission(&self, chunk: &ChunkPacked, position: ChunkPosition) -> u4;
	fn initial(&self, chunk: &ChunkPacked, data: &mut ChunkNibbles, mask: &mut SpillChunkMask);
}

#[derive(Debug)]
pub struct BlockLightSources {
	emission: BulkNibbles
}

impl BlockLightSources {
	pub fn new(palette_bits: usize) -> Self {
		BlockLightSources {
			emission: BulkNibbles::new(1 << palette_bits)
		}
	}
	
	pub fn set_emission(&mut self, raw_index: usize, value: u4) {
		self.emission.set(raw_index, value)
	}
}

impl LightSources for BlockLightSources {
	fn emission(&self, chunk: &ChunkPacked, position: ChunkPosition) -> u4 {
		self.emission.get(chunk.get(position) as usize)
	}
	
	fn initial(&self, _chunk: &ChunkPacked, _data: &mut ChunkNibbles, _mask: &mut SpillChunkMask) {
		unimplemented!()
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SkyLightSources {
	heightmap: ChunkHeightMap
}

impl SkyLightSources {
	pub fn build(chunk: &ChunkPacked, matches: &BitVec, no_light: LayerMask) -> Self {
		SkyLightSources {
			heightmap: ChunkHeightMap::build(chunk, matches, no_light)
		}
	}
	
	pub fn heightmap(&self) -> &LayerNibbles {
		self.heightmap.heightmap()
	}

	fn no_light(&self) -> &LayerMask {
		self.heightmap.is_filled()
	} 

	pub fn into_mask(self) -> LayerMask {
		self.heightmap.into_mask()
	}
}

impl LightSources for SkyLightSources {
	fn emission(&self, _: &ChunkPacked, position: ChunkPosition) -> u4 {
		// no_light -> height of 16 or more
		let height = ((self.no_light()[position.layer()] as u8) << 4) | self.heightmap().get(position.layer()).raw();

		u4::new(if position.y() >= height { 15 } else { 0 })
	}
	
	fn initial(&self, _: &ChunkPacked, data: &mut ChunkNibbles, mask: &mut SpillChunkMask) {
		if self.no_light().is_filled(true) {
			// Note: This assumes that the chunk is already filled with zeros...

			// Skip lighting entirely, as there is no light in this chunk.
			return;
		}

		let mut max_heightmap = 0;

		// Check to see if every ZX coordinate has a sky light source.
		// If this is true, there are 2 possible optimizations:
		//
		// First: Not only does every ZX coordinate have a sky light source, the chunk is entirely filled with light.
		// In this case, no queueing is needed inside the chunk, but the horizontal and down sides need to be queued for checking.
		//
		// Second: If there are some blocks blocking sky light, there may be a volume of 16x?x16 that contains level 15 sky light.
		// This presents a simplified form of queueing, as only blocks at the edge of the volume need to be queued for checking.

		if self.no_light().is_filled(false) {
			if self.heightmap().is_filled(u4::new(0)) {
				data.fill(u4::new(15));

				mask.spills[dir::Down].fill(true);
				mask.spills[dir::PlusX].fill(true);
				mask.spills[dir::MinusX].fill(true);
				mask.spills[dir::PlusZ].fill(true);
				mask.spills[dir::MinusZ].fill(true);

				// The chunk is entirely filled with light.
				return;
			}

			// The chunk is partially lit at every layer position by skylight, allowing optimizations.
			// First, determine the maximum value in the heightmap.
			// This is the Y value where it is safe to fill it and above with 100% light.

			for position in LayerPosition::enumerate() {
				max_heightmap = max(max_heightmap, self.heightmap().get(position).raw());
			}

			// Fill the common area between all of the height maps.

			for y in max_heightmap..16 {
				for position in LayerPosition::enumerate() {
					data.set(ChunkPosition::from_layer(y, position), u4::new(15));
				}
			}

			// Enqueue blocks on the PlusX and MinusX faces, using ZY coordinates.
			for z in 0..16 {
				for y in max_heightmap..16 {
					let layer = LayerPosition::new(y, z);

					mask.spills[dir::PlusX].set_true(layer);
					mask.spills[dir::MinusX].set_true(layer);
				}
			}

			// Enqueue blocks on the PlusZ and MinusZ faces, using XY coordinates.
			for y in max_heightmap..16 {
				for x in 0..16 {
					let layer = LayerPosition::new(x, y);

					mask.spills[dir::PlusZ].set_true(layer);
					mask.spills[dir::MinusZ].set_true(layer);
				}
			}

			// Note: queueing blocks on the Down face is handled by the loop below.
			// Queuing blocks on the Up face is not necessary, because the block above has to let skylight through.
		} else {
			// Same behavior as optimization disabled.
			max_heightmap = 16;
		}

		// Slowest part: Fill in the irregular part of the terrain with the remaining light sources.
		// This is the source of most of the queueing, but the optimizations remaining are most likely slim.

		for position in LayerPosition::enumerate() {
			if self.no_light()[position] {
				continue;
			}

			let lowest = self.heightmap().get(position).raw();

			// We do not need to enqueue the block in the upper direction, as it is already the maximum light value.
			// But, we need to enqueue the block below the heightmap value.

			mask.set_offset_true(ChunkPosition::from_layer(lowest, position), dir::Down);

			for y in lowest..max_heightmap {
				let position = ChunkPosition::from_layer(y, position);

				data.set(position, u4::new(15));

				mask.set_offset_true(position, dir::MinusX);
				mask.set_offset_true(position, dir::MinusZ);
				mask.set_offset_true(position, dir::PlusX);
				mask.set_offset_true(position, dir::PlusZ);
			}
		}
	}
}
