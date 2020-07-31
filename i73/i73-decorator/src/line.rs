use std::cmp;
use vocs::position::QuadPosition;

// TODO: This should be close enough, but is unverified.

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Line {
	pub from: QuadPosition,
	pub to: QuadPosition,
}

impl Line {
	/// Offset that needs to be applied to `from` to get `to`.
	pub fn offset(&self) -> (i8, i8, i8) {
		(
			(self.to.x() as i8) - (self.from.x() as i8),
			(self.to.y() as i8) - (self.from.y() as i8),
			(self.to.z() as i8) - (self.from.z() as i8),
		)
	}

	pub fn trace(&self) -> LineTracer {
		let diff = self.offset();

		let max = cmp::max(diff.0.abs(), cmp::max(diff.1.abs(), diff.2.abs()));

		LineTracer {
			steps: max as u32,
			iterations: 0,
			velocity: (
				(diff.0 as f64) / (max as f64),
				(diff.1 as f64) / (max as f64),
				(diff.2 as f64) / (max as f64),
			),
			position: (self.from.x() as f64, self.from.y() as f64, self.from.z() as f64),
		}
	}
}

pub struct LineTracer {
	velocity: (f64, f64, f64),
	position: (f64, f64, f64),
	steps: u32,
	iterations: u32,
}

impl Iterator for LineTracer {
	type Item = QuadPosition;

	fn next(&mut self) -> Option<Self::Item> {
		if self.iterations >= self.steps {
			return None;
		}

		let position = [
			self.position.0 + self.velocity.0,
			self.position.1 + self.velocity.1,
			self.position.2 + self.velocity.2,
		];

		self.position = (position[0], position[1], position[2]);
		let position = QuadPosition::new(
			(position[0] + 0.5).floor() as u8,
			(position[1] + 0.5).floor() as u8,
			(position[2] + 0.5).floor() as u8,
		);

		self.iterations += 1;

		Some(position)
	}
}
