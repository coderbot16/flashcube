use cgmath::{Point2, Vector2};
use vocs::position::LayerPosition;
use vocs::unpacked::Layer;

pub trait Sample {
	type Output: Default + Copy;

	/// Coordinates are in block space
	fn sample(&self, point: Point2<f64>) -> Self::Output;

	/// An optimized version of this function is usually provided by the implementor.
	fn chunk(&self, chunk: (f64, f64)) -> Layer<Self::Output> {
		let mut out = Layer::filled(Self::Output::default());
		let chunk = Point2::new(chunk.0, chunk.1);

		for position in LayerPosition::enumerate() {
			let point = chunk + Vector2::new(position.x() as f64, position.z() as f64);

			out[position] = self.sample(point);
		}

		out
	}
}
