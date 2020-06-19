use crate::queue::Queue;
use crate::sources::LightSources;
use vocs::nibbles::{u4, ChunkNibbles, BulkNibbles};
use vocs::packed::ChunkPacked;
use vocs::component::ChunkStorage;
use vocs::position::{ChunkPosition, Offset, dir};
use vocs::view::Directional;
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
	
	pub fn initial(&mut self, queue: &mut Queue) {
		self.sources.initial(&mut self.data, queue.mask_mut())
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
				self.sources.emission(at)
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
