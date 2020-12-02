use cgmath::{Point2, Vector2};
use i73_base::Layer;
use vocs::position::LayerPosition;

pub trait Sample {
	type Output: Default + Copy;

	/// Coordinates are in block space
	fn sample(&self, point: Point2<f64>) -> Self::Output;

	/// An optimized version of this function is usually provided by the implementor.
	fn chunk(&self, chunk: (f64, f64)) -> Layer<Self::Output> {
		let mut out = Layer::fill(Self::Output::default());
		let chunk = Point2::new(chunk.0, chunk.1);

		for position in LayerPosition::enumerate() {
			let point = chunk + Vector2::new(position.x() as f64, position.z() as f64);

			out.set(position, self.sample(point));
		}

		out
	}
}
