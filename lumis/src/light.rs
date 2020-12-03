use crate::queue::CubeQueue;
use crate::sources::LightSources;
use std::cmp::max;
use vocs::component::CubeStorage;
use vocs::nibbles::{u4, NibbleArray, NibbleCube};
use vocs::packed::PackedCube;
use vocs::position::{dir, CubePosition, Offset};
use vocs::view::Directional;

#[derive(Debug)]
pub struct Lighting<'n, S>
where
	S: LightSources,
{
	data: &'n mut NibbleCube,
	neighbors: Directional<&'n NibbleCube>,
	sources: S,
	opacity: NibbleArray,
}

impl<'n, S> Lighting<'n, S>
where
	S: LightSources,
{
	pub fn new(
		data: &'n mut NibbleCube, neighbors: Directional<&'n NibbleCube>, sources: S,
		opacity: NibbleArray,
	) -> Self {
		Lighting { data, neighbors, sources, opacity }
	}

	fn get(&self, at: CubePosition) -> u4 {
		self.data.get(at)
	}

	pub fn initial(&mut self, queue: &mut CubeQueue) {
		self.sources.initial(&mut self.data, queue.mask_mut())
	}

	fn update(&mut self, queue: &mut CubeQueue, at: CubePosition, opacity: u4) {
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

	pub fn apply(&mut self, chunk: &PackedCube, queue: &mut CubeQueue) {
		while queue.flip() {
			while let Some(at) = queue.pop_first() {
				let opacity = self.opacity.get(chunk.get(at) as usize);

				self.update(queue, at, opacity);
			}
		}
	}

	pub fn decompose(self) -> (&'n mut NibbleCube, S) {
		(self.data, self.sources)
	}
}
