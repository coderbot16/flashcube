extern crate cgmath;
extern crate i73_base;
extern crate i73_noise;
extern crate java_rand;
extern crate vocs;

pub mod climate;
pub mod segmented;
pub mod source;

use climate::Climate;
use i73_base::{Block, Layer};
use segmented::Segmented;
use std::borrow::Cow;
use vocs::indexed::{LayerIndexed, Target};
use vocs::position::LayerPosition;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Biome {
	pub surface: Surface,
	pub name: Cow<'static, str>,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Surface {
	pub top: Block,
	pub fill: Block,
	pub chain: Vec<Followup>,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Followup {
	pub block: Block,
	pub max_depth: u32,
}

#[derive(Debug)]
pub struct Grid<B: Clone>(pub Segmented<Segmented<B>>);
impl<B: Clone> Grid<B> {
	fn new_temperatures(biome: B) -> Segmented<B> {
		let mut temperatures = Segmented::new(biome.clone());
		temperatures.add_boundary(1.0, biome.clone());

		temperatures
	}

	pub fn new(default: B) -> Self {
		let temperatures = Self::new_temperatures(default);

		let mut grid = Segmented::new(temperatures.clone());
		grid.add_boundary(1.0, temperatures.clone());

		Grid(grid)
	}

	pub fn add(&mut self, temperature: (f64, f64), rainfall: (f64, f64), biome: B) {
		self.0.for_all_aligned(
			rainfall.0,
			rainfall.1,
			&|| Self::new_temperatures(biome.clone()),
			&|temperatures| {
				temperatures.for_all_aligned(
					temperature.0,
					temperature.1,
					&|| biome.clone(),
					&|existing| {
						*existing = biome.clone();
					},
				)
			},
		)
	}

	pub fn lookup(&self, climate: Climate) -> &B {
		self.0.get(climate.adjusted_rainfall()).get(climate.temperature())
	}
}

pub struct Lookup<B: Clone>(Box<[B]>);
impl<B: Clone> Lookup<B> {
	pub fn filled(biome: &B) -> Self {
		let mut lookup = Vec::with_capacity(4096);

		for _ in 0..4096 {
			lookup.push(biome.clone());
		}

		Lookup(lookup.into_boxed_slice())
	}

	pub fn generate(grid: &Grid<B>) -> Self {
		let mut lookup = Vec::with_capacity(4096);

		for index in 0..4096 {
			let (temperature, rainfall) = (index / 64, index % 64);

			let climate = Climate::new((temperature as f64) / 63.0, (rainfall as f64) / 63.0);

			lookup.push(grid.lookup(climate).clone());
		}

		Lookup(lookup.into_boxed_slice())
	}

	pub fn lookup_raw(&self, temperature: usize, rainfall: usize) -> &B {
		&self.0[temperature * 64 + rainfall]
	}

	pub fn lookup(&self, climate: Climate) -> &B {
		self.lookup_raw(
			(climate.temperature() * 63.0) as usize,
			(climate.rainfall() * 63.0) as usize,
		)
	}
}

impl<B: Target> Lookup<B> {
	pub fn climates_to_biomes(&self, climates: &Layer<Climate>) -> LayerIndexed<B> {
		// TODO: Avoid the default lookup and clone.
		let mut layer = LayerIndexed::new(2, self.lookup(Climate::alpha()).clone());

		for position in LayerPosition::enumerate() {
			let climate = climates.get(position);
			let biome = self.lookup(climate);

			layer.set_immediate(position, biome);
		}

		layer
	}
}
