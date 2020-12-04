use crate::queue::CubeQueue;
use crate::sources::LightSources;
use crate::PackedNibbleCube;
use std::cmp::max;
use vocs::nibbles::{u4, NibbleArray};
use vocs::packed::PackedCube;
use vocs::position::{dir, CubePosition, Offset};
use vocs::view::Directional;

#[derive(Debug)]
pub struct Lighting<'n, S>
where
	S: LightSources,
{
	data: &'n mut PackedNibbleCube,
	neighbors: Directional<&'n PackedNibbleCube>,
	sources: S,
	opacity: NibbleArray,
}

impl<'n, S> Lighting<'n, S>
where
	S: LightSources,
{
	pub fn new(
		data: &'n mut PackedNibbleCube, neighbors: Directional<&'n PackedNibbleCube>, sources: S,
		opacity: NibbleArray,
	) -> Self {
		Lighting { data, neighbors, sources, opacity }
	}

	fn get(&self, at: CubePosition) -> u4 {
		self.data.get(at)
	}

	fn update(&mut self, blocks: &PackedCube, queue: &mut CubeQueue, at: CubePosition, opacity: u4) {
		let emission = self.sources.emission(blocks, at);

		// Fast path: No need to pull in light from neighbors here, if the max light value is
		// already emitted at this location
		//
		// This shaves off a quite significant amount of time from light computation because it
		// allows us to avoid the gigantic light propagation logic below
		if emission == u4::MAX {
			if self.data.get(at) != u4::MAX {
				self.data.set(at, u4::MAX);
				queue.enqueue_neighbors(at);
			}

			return;
		}

		// In addition to the base opacity, we also subtract 1 light level for each block travelled
		// (in Manhattan distance)
		let opacity = opacity.saturating_add(u4::ONE);

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

		let light = max(
			max_value.saturating_sub(opacity), 
			emission
		);

		if light != self.data.get(at) {
			self.data.set(at, light);
			queue.enqueue_neighbors(at);
		}
	}

	pub fn apply(&mut self, chunk: &PackedCube, queue: &mut CubeQueue) {
		while queue.flip() {
			while let Some(at) = queue.pop_first() {
				let opacity = self.opacity.get(chunk.get(at) as usize);

				self.update(chunk, queue, at, opacity);
			}
		}
	}

	pub fn decompose(self) -> (&'n mut PackedNibbleCube, S) {
		(self.data, self.sources)
	}
}
