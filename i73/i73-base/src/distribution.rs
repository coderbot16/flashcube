use java_rand::Random;

/// A random distribution.
pub trait Distribution {
	fn next(&self, rng: &mut Random) -> u32;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChanceOrdering {
	AlwaysGeneratePayload,
	CheckChanceBeforePayload,
}

#[derive(Debug)]
pub struct Chance<D>
where
	D: Distribution,
{
	/// Chance for this distribution to return its value instead of 0.
	/// Represented as probability = 1 / chance.
	/// A chance of "1" does not call the Chance RNG, and acts as if it passed.
	pub chance: u32,
	pub ordering: ChanceOrdering,
	pub base: D,
}

impl<D> Distribution for Chance<D>
where
	D: Distribution,
{
	fn next(&self, rng: &mut Random) -> u32 {
		match self.ordering {
			ChanceOrdering::AlwaysGeneratePayload => {
				let payload = self.base.next(rng);

				if self.chance <= 1 {
					payload
				} else if rng.next_u32_bound(self.chance) == 0 {
					payload
				} else {
					0
				}
			}
			ChanceOrdering::CheckChanceBeforePayload => {
				if self.chance <= 1 {
					self.base.next(rng)
				} else if rng.next_u32_bound(self.chance) == 0 {
					self.base.next(rng)
				} else {
					0
				}
			}
		}
	}
}

/// Baseline distribution. This should be general enough to fit most use cases.
#[derive(Debug)]
pub enum Baseline {
	Constant { value: u32 },
	Linear(Linear),
	Packed2(Packed2),
	Packed3(Packed3),
	Centered(Centered),
}

impl Distribution for Baseline {
	fn next(&self, rng: &mut Random) -> u32 {
		match *self {
			Baseline::Constant { value } => value,
			Baseline::Linear(ref linear) => linear.next(rng),
			Baseline::Packed2(ref packed2) => packed2.next(rng),
			Baseline::Packed3(ref packed3) => packed3.next(rng),
			Baseline::Centered(ref centered) => centered.next(rng),
		}
	}
}

impl Distribution for u32 {
	fn next(&self, _: &mut Random) -> u32 {
		*self
	}
}

/// Plain old linear distribution, with a minimum and maximum.
#[derive(Debug)]
pub struct Linear {
	pub min: u32,
	pub max: u32,
}

impl Distribution for Linear {
	fn next(&self, rng: &mut Random) -> u32 {
		self.min + rng.next_u32_bound(self.max - self.min + 1)
	}
}

/// Distribution that packs more values to the minimum value. This is based on 2 RNG iterations.
#[derive(Debug)]
pub struct Packed2 {
	pub min: u32,
	/// Minimum height passed to the second RNG call (the linear call).
	pub linear_start: u32,
	pub max: u32,
}

impl Distribution for Packed2 {
	fn next(&self, rng: &mut Random) -> u32 {
		let initial = rng.next_u32_bound(self.max - self.linear_start + 2);

		self.min + rng.next_u32_bound(initial + self.linear_start - self.min)
	}
}

/// Distribution that packs more values to the minimum value. This is based on 3 RNG iterations, and is more extreme.
/// The average is around `(max+1)/8 - 1`, a simplified form of `(max+1)/2?? - 1`.
#[derive(Debug)]
pub struct Packed3 {
	pub max: u32,
}

impl Distribution for Packed3 {
	fn next(&self, rng: &mut Random) -> u32 {
		let result = rng.next_u32_bound(self.max + 1);
		let result = rng.next_u32_bound(result + 1);
		rng.next_u32_bound(result + 1)
	}
}

/// Distribution centered around a certain point, with a maximum variance.
#[derive(Debug)]
pub struct Centered {
	pub center: u32,
	pub radius: u32,
}

impl Distribution for Centered {
	fn next(&self, rng: &mut Random) -> u32 {
		rng.next_u32_bound(self.radius) + rng.next_u32_bound(self.radius) + self.center
			- self.radius
	}
}
