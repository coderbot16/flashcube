use crate::queue::ChunkQueue;
use crate::sources::LightSources;
use std::cmp::max;
use vocs::component::ChunkStorage;
use vocs::nibbles::{u4, BulkNibbles, ChunkNibbles};
use vocs::packed::ChunkPacked;
use vocs::position::{dir, ChunkPosition, Offset};
use vocs::view::Directional;

#[derive(Debug)]
pub struct Lighting<'n, S>
where
	S: LightSources,
{
	data: &'n mut ChunkNibbles,
	neighbors: Directional<&'n ChunkNibbles>,
	sources: S,
	opacity: BulkNibbles,
}

impl<'n, S> Lighting<'n, S>
where
	S: LightSources,
{
	pub fn new(
		data: &'n mut ChunkNibbles, neighbors: Directional<&'n ChunkNibbles>, sources: S,
		opacity: BulkNibbles,
	) -> Self {
		Lighting { data, neighbors, sources, opacity }
	}

	fn get(&self, at: ChunkPosition) -> u4 {
		self.data.get(at)
	}

	pub fn initial(&mut self, queue: &mut ChunkQueue) {
		self.sources.initial(&mut self.data, queue.mask_mut())
	}

	fn update(&mut self, queue: &mut ChunkQueue, at: ChunkPosition, opacity: u4) {
		let max_value = max(
			max(
				max(
					at.offset(dir::MinusX)
						.map(|at| self.get(at))
						.unwrap_or_else(|| self.neighbors[dir::MinusX].get(at.with_x(15))),
					at.offset(dir::PlusX)
						.map(|at| self.get(at))
						.unwrap_or_else(|| self.neighbors[dir::PlusX].get(at.with_x(0))),
				),
				max(
					at.offset(dir::MinusZ)
						.map(|at| self.get(at))
						.unwrap_or_else(|| self.neighbors[dir::MinusZ].get(at.with_z(15))),
					at.offset(dir::PlusZ)
						.map(|at| self.get(at))
						.unwrap_or_else(|| self.neighbors[dir::PlusZ].get(at.with_z(0))),
				),
			),
			max(
				at.offset(dir::Down)
					.map(|at| self.get(at))
					.unwrap_or_else(|| self.neighbors[dir::Down].get(at.with_y(15))),
				at.offset(dir::Up)
					.map(|at| self.get(at))
					.unwrap_or_else(|| self.neighbors[dir::Up].get(at.with_y(0))),
			),
		);

		let light = max(max_value.saturating_sub(u4::new(1)), self.sources.emission(at))
			.saturating_sub(opacity);

		if light != self.data.get(at) {
			self.data.set(at, light);
			queue.enqueue_neighbors(at);
		}
	}

	pub fn apply(&mut self, chunk: &ChunkPacked, queue: &mut ChunkQueue) {
		while queue.flip() {
			while let Some(at) = queue.pop_first() {
				let opacity = self.opacity.get(chunk.get(at) as usize);

				self.update(queue, at, opacity);
			}
		}
	}

	pub fn decompose(self) -> (&'n mut ChunkNibbles, S) {
		(self.data, self.sources)
	}
}