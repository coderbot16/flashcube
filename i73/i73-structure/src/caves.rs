use i73_base::distribution::{Chance, ChanceOrdering, Distribution, Linear, Packed2, Packed3};
use i73_base::matcher::BlockMatcher;
use i73_base::{math, Block};
use i73_trig as trig;
use java_rand::Random;
use std::cmp::{max, min};
use vocs::position::{ColumnPosition, GlobalColumnPosition};
use vocs::view::{ColumnAssociation, ColumnBlocks, ColumnMut, ColumnPalettes};
use StructureGenerator;

const NOTCH_PI: f32 = 3.141593;
const PI_DIV_2: f32 = 1.570796;
const MIN_H_SIZE: f64 = 1.5;

/// Make many chunks not spawn cave starts at all, otherwise the world would look like swiss cheese.
/// Note that caves starting in other chunks can still carve through this chunk.
/// Offsets the fact that a single cave start can branch many times.
/// Also make most chunks that do contain caves contain few, but have the potential to contain many.
pub static RARITY: Chance<Packed3> = Chance {
	base: Packed3 { max: 39 },
	chance: 15,
	ordering: ChanceOrdering::AlwaysGeneratePayload,
};

/// Allow caves at high altitudes, but make most of them spawn underground.
pub static HEIGHT: Packed2 = Packed2 { min: 0, linear_start: 8, max: 126 };

/// More chunks will have cave starts, but they will have less in each one.
/// Results in less caves overall, since a chunk is 3x more likely to have cave starts,
/// but will have a maximum that is 4x less.
pub static RARITY_NETHER: Chance<Packed3> =
	Chance { base: Packed3 { max: 9 }, chance: 5, ordering: ChanceOrdering::AlwaysGeneratePayload };

/// Since the Nether has a high amount of solid blocks from bottom to top, caves spawn uniformly.
pub static HEIGHT_NETHER: Linear = Linear { min: 0, max: 127 };

struct CavesAssociations {
	carve: ColumnAssociation,
	lower: ColumnAssociation,
	surface: ColumnAssociation,
}

// Overworld: CavesGenerator { carve: air, ocean: [ flowing_water, still_water ], carvable: [ stone, dirt, grass ], spheroid_size_multiplier: 1.0, vertical_multiplier: 1.0 }
// Nether: CavesGenerator { carve: air, ocean: [ flowing_lava, still_lava ], carvable: [ netherrack, dirt, grass ], spheroid_size_multiplier: 2.0, vertical_multiplier: 0.5}

pub struct CavesGenerator {
	pub carve: Block,
	pub lower: Block,
	pub surface_block: Block,
	pub ocean: BlockMatcher,
	pub surface_top: BlockMatcher,
	pub surface_fill: BlockMatcher,
	pub carvable: BlockMatcher,
	pub spheroid_size_multiplier: f32,
	pub vertical_multiplier: f64,
	pub lower_surface: u8,
}

impl CavesGenerator {
	fn carve_spheroid(
		&self, spheroid: Spheroid, associations: &CavesAssociations, blocks: &mut ColumnBlocks,
		palette: &ColumnPalettes<Block>, chunk: GlobalColumnPosition,
	) {
		let chunk_block = ((chunk.x() * 16) as f64, (chunk.z() * 16) as f64);

		// Try to make sure that we don't carve into the ocean.
		// However, this misses chunk boundaries - there is no easy way to fix this.

		let y_top = spheroid.upper.y() + 2;
		let y_bottom = spheroid.lower.y() - 1;

		let check_water =
			|x, y, z| self.ocean.matches(blocks.get(ColumnPosition::new(x, y, z), palette));

		for z in spheroid.lower.z()..=spheroid.upper.z() {
			for x in spheroid.lower.x()..=spheroid.upper.x() {
				let edge = z == spheroid.lower.z()
					|| z == spheroid.upper.z()
					|| x == spheroid.lower.x()
					|| x == spheroid.upper.x();

				if !edge {
					if check_water(x, y_top, z) {
						return;
					}
					if check_water(x, y_bottom, z) {
						return;
					}
				} else {
					for y in ((spheroid.lower.y() - 1)..=y_top).rev() {
						if check_water(x, y, z) {
							return;
						}
					}
				}
			}
		}

		// TODO: FloorY
		// block.1 > (-0.7) * spheroid.size.vertical + spheroid.center.1 - 0.5

		for z in spheroid.lower.z()..=spheroid.upper.z() {
			for x in spheroid.lower.x()..=spheroid.upper.x() {
				let mut hit_surface_top = false;

				// Need to go downwards so that the grass gets pulled down.
				for y in (spheroid.lower.y()..=spheroid.upper.y()).rev() {
					let position = ColumnPosition::new(x, y, z);

					let block = (x as f64, y as f64, z as f64);

					let scaled = (
						(block.0 + chunk_block.0 + 0.5 - spheroid.center.0)
							/ spheroid.size.horizontal,
						(block.1 + 0.5 - spheroid.center.1) / spheroid.size.vertical,
						(block.2 + chunk_block.1 + 0.5 - spheroid.center.2)
							/ spheroid.size.horizontal,
					);

					// TODO: Grass pulldown sometimes is inconsistent?

					// Test if the block is within the spheroid region. Additionally, the y > -0.7 check makes the floors flat.
					if scaled.1 > -0.7
						&& scaled.0 * scaled.0 + scaled.1 * scaled.1 + scaled.2 * scaled.2 < 1.0
					{
						let block = blocks.get(position, palette);

						if self.surface_top.matches(block) {
							hit_surface_top = true;
						}

						if !self.carvable.matches(block) && !self.ocean.matches(block) {
							continue;
						}

						if y < self.lower_surface {
							blocks.set(position, &associations.lower);
						} else {
							blocks.set(position, &associations.carve);

							if y > 0 && hit_surface_top {
								let below = ColumnPosition::new(x, y - 1, z);

								if self.surface_fill.matches(blocks.get(below, palette)) {
									blocks.set(below, &associations.surface);
								}
							}
						}
					}
				}
			}
		}
	}

	fn carve_tunnel(
		&self, mut tunnel: Tunnel, caves: &mut Caves, associations: &CavesAssociations,
		blocks: &mut ColumnBlocks, palette: &ColumnPalettes<Block>, chunk: GlobalColumnPosition,
		from: GlobalColumnPosition, radius: u32,
	) {
		loop {
			let outcome = tunnel.step(self.vertical_multiplier);

			match outcome {
				Outcome::Split => {
					let (a, b) = tunnel.split(caves);

					self.carve_tunnel(a, caves, associations, blocks, palette, chunk, from, radius);
					self.carve_tunnel(b, caves, associations, blocks, palette, chunk, from, radius);

					return;
				}
				Outcome::Constrict => (),
				Outcome::Unreachable => return,
				Outcome::OutOfChunk => (),
				Outcome::Carve(Some(spheroid)) => {
					self.carve_spheroid(spheroid, associations, blocks, palette, chunk)
				}
				Outcome::Carve(None) => (),
				Outcome::Done => return,
			}
		}
	}
}

impl StructureGenerator for CavesGenerator {
	fn generate(
		&self, random: Random, column: &mut ColumnMut<Block>, chunk: GlobalColumnPosition,
		from: GlobalColumnPosition, radius: u32,
	) {
		let mut caves =
			Caves::for_chunk(random, chunk, from, radius, self.spheroid_size_multiplier);

		column.ensure_available(self.carve.clone());
		column.ensure_available(self.lower.clone());
		column.ensure_available(self.surface_block.clone());

		let (mut blocks, palette) = column.freeze_palette();

		let associations = CavesAssociations {
			carve: palette.reverse_lookup(&self.carve).unwrap(),
			lower: palette.reverse_lookup(&self.lower).unwrap(),
			surface: palette.reverse_lookup(&self.surface_block).unwrap(),
		};

		while let Some(start) = caves.next() {
			match start {
				Start::Tunnel(tunnel) => self.carve_tunnel(
					tunnel,
					&mut caves,
					&associations,
					&mut blocks,
					&palette,
					chunk,
					from,
					radius,
				),
				Start::Circular(Some(spheroid)) => {
					self.carve_spheroid(spheroid, &associations, &mut blocks, &palette, chunk)
				}
				Start::Circular(None) => (),
			};
		}
	}
}

#[derive(Debug)]
pub struct Caves {
	state: Random,
	chunk: GlobalColumnPosition,
	from: GlobalColumnPosition,
	remaining: u32,
	max_chunk_radius: u32,
	spheroid_size_multiplier: f32,
	extra: Option<(u32, (f64, f64, f64))>,
}

impl Caves {
	pub fn for_chunk(
		mut state: Random, chunk: GlobalColumnPosition, from: GlobalColumnPosition, radius: u32,
		spheroid_size_multiplier: f32,
	) -> Caves {
		let remaining = RARITY.next(&mut state);

		Caves {
			state,
			chunk,
			from,
			remaining,
			extra: None,
			max_chunk_radius: radius,
			spheroid_size_multiplier,
		}
	}
}

impl Iterator for Caves {
	type Item = Start;

	fn next(&mut self) -> Option<Start> {
		if self.remaining == 0 {
			return None;
		}

		self.remaining -= 1;

		if let &mut Some((ref mut extra, orgin)) = &mut self.extra {
			if *extra > 0 {
				*extra -= 1;

				return Some(Start::normal(
					&mut self.state,
					self.chunk,
					orgin,
					self.max_chunk_radius,
					self.spheroid_size_multiplier,
				));
			}
		}

		self.extra = None;

		let x = self.state.next_i32_bound(16);
		let mut y = self.state.next_u32_bound(120);
		y = self.state.next_u32_bound(y + 8);
		let z = self.state.next_i32_bound(16);

		let orgin = ((self.from.x() * 16 + x) as f64, y as f64, (self.from.z() * 16 + z) as f64);

		if self.state.next_u32_bound(4) == 0 {
			let circular =
				Start::circular(&mut self.state, self.chunk, orgin, self.max_chunk_radius);
			let extra = 1 + self.state.next_u32_bound(4);

			self.remaining += extra;
			self.extra = Some((extra, orgin));

			Some(circular)
		} else {
			Some(Start::normal(
				&mut self.state,
				self.chunk,
				orgin,
				self.max_chunk_radius,
				self.spheroid_size_multiplier,
			))
		}
	}
}

#[derive(Debug)]
pub enum Start {
	Circular(Option<Spheroid>),
	Tunnel(Tunnel),
}

impl Start {
	fn normal(
		rng: &mut Random, chunk: GlobalColumnPosition, block: (f64, f64, f64),
		max_chunk_radius: u32, spheroid_size_multiplier: f32,
	) -> Self {
		Start::Tunnel(Tunnel::normal(rng, chunk, block, max_chunk_radius, spheroid_size_multiplier))
	}

	fn circular(
		rng: &mut Random, chunk: GlobalColumnPosition, block: (f64, f64, f64),
		max_chunk_radius: u32,
	) -> Self {
		let spheroid_size_factor = 1.0 + rng.next_f32() * 6.0;
		let mut state = Random::new(rng.next_u64());

		let mut size = SystemSize::new(&mut state, 0, max_chunk_radius);
		size.current = size.max / 2;

		let size = SpheroidSize::from_horizontal(
			MIN_H_SIZE
				+ (trig::sin(size.current as f32 * NOTCH_PI / size.max as f32)
					* spheroid_size_factor) as f64,
			0.5,
		);

		let position = Position::new(chunk, (block.0 + 1.0, block.1, block.2));

		Start::Circular(if position.out_of_chunk(&size) { None } else { position.spheroid(size) })
	}
}

#[derive(Debug)]
pub struct Tunnel {
	state: Random,
	position: Position,
	size: SystemSize,
	split: Option<u32>,
	/// 0.92 = Steep, 0.7 = Normal
	pitch_keep: f32,
	spheroid_size_factor: f32,
}

impl Tunnel {
	fn normal(
		rng: &mut Random, chunk: GlobalColumnPosition, block: (f64, f64, f64),
		max_chunk_radius: u32, spheroid_size_multiplier: f32,
	) -> Self {
		let position = Position::with_angles(
			chunk,
			block,
			rng.next_f32() * NOTCH_PI * 2.0,
			(rng.next_f32() - 0.5) / 4.0,
		);
		let spheroid_size_factor =
			(rng.next_f32() * 2.0 + rng.next_f32()) * spheroid_size_multiplier;

		let mut state = Random::new(rng.next_u64());

		let size = SystemSize::new(&mut state, 0, max_chunk_radius);

		Tunnel {
			position,
			size,
			split: size.split(&mut state, spheroid_size_factor),
			pitch_keep: if state.next_u32_bound(6) == 0 { 0.92 } else { 0.7 },
			spheroid_size_factor,
			state,
		}
	}

	fn split_off(&mut self, rng: &mut Random, yaw_offset: f32) -> Tunnel {
		let position = Position::with_angles(
			self.position.chunk,
			(self.position.x, self.position.y, self.position.z),
			self.position.yaw + yaw_offset,
			self.position.pitch / 3.0,
		);
		let spheroid_size_factor = self.state.next_f32() * 0.5 + 0.5;

		let mut state = Random::new(rng.next_u64());

		let size = self.size;

		Tunnel {
			position,
			size,
			split: size.split(&mut state, spheroid_size_factor),
			pitch_keep: if state.next_u32_bound(6) == 0 { 0.92 } else { 0.7 },
			spheroid_size_factor,
			state,
		}
	}

	fn split(&mut self, caves: &mut Caves) -> (Tunnel, Tunnel) {
		// https://bugs.mojang.com/browse/MC-7196
		// First bug resulting in chunk cliffs, that we have to recreate
		// The tunnel splitting calls back to the root RNG, causing discontinuities if is_chunk_unreachable() returns true before the cave splits.
		// If the is_chunk_unreachable optimization is disabled, this issue doesn't occur.
		// It also wrecks the nice, clean iterator implementation, as we have to pass the RNG down. Ugh.

		(self.split_off(&mut caves.state, -PI_DIV_2), self.split_off(&mut caves.state, PI_DIV_2))
	}

	fn is_chunk_unreachable(&self) -> bool {
		// https://bugs.mojang.com/browse/MC-7200
		// Second bug resulting in chunk cliffs, that we have to recreate.
		// Using addition/subtraction with distance squared math is invalid.

		let remaining = (self.size.max - self.size.current) as f64;

		// Conservative buffer distance that accounts for the size of each carved part.
		let buffer = (self.spheroid_size_factor * 2.0 + 16.0) as f64;

		// Invalid: Subtraction from distance squared.
		self.position.distance_from_chunk_squared() - remaining * remaining > buffer * buffer
	}

	fn next_spheroid_size(&self) -> f64 {
		MIN_H_SIZE
			+ (trig::sin(self.size.current as f32 * NOTCH_PI / self.size.max as f32)
				* self.spheroid_size_factor) as f64
	}

	pub fn step(&mut self, vertical_multiplier: f64) -> Outcome {
		if self.size.done() {
			return Outcome::Done;
		}

		self.position.step(&mut self.state, self.pitch_keep);

		if self.size.should_split(self.split) {
			return Outcome::Split;
		}

		if self.state.next_u32_bound(4) == 0 {
			self.size.step();
			return Outcome::Constrict;
		}

		if self.is_chunk_unreachable() {
			return Outcome::Unreachable;
		}

		let size = SpheroidSize::from_horizontal(self.next_spheroid_size(), vertical_multiplier);

		if self.position.out_of_chunk(&size) {
			self.size.step();
			return Outcome::OutOfChunk;
		}

		let spheroid = self.position.spheroid(size);

		self.size.step();

		Outcome::Carve(spheroid)
	}
}

#[derive(Debug, Copy, Clone)]
struct SystemSize {
	current: u32,
	max: u32,
}

impl SystemSize {
	fn new(rng: &mut Random, current: u32, max_chunk_radius: u32) -> Self {
		let max_block_radius = max_chunk_radius * 16 - 16;
		let max = max_block_radius - rng.next_u32_bound(max_block_radius / 4);

		SystemSize { current, max }
	}

	pub fn step(&mut self) {
		self.current += 1;
	}

	pub fn done(&self) -> bool {
		self.current >= self.max
	}

	pub fn should_split(&self, split_threshold: Option<u32>) -> bool {
		Some(self.current) == split_threshold
	}

	/// Returns the point where the tunnel will split into 2. Returns None if it won't split.
	fn split(&self, rng: &mut Random, spheroid_size_factor: f32) -> Option<u32> {
		let split = rng.next_u32_bound(self.max / 2) + self.max / 4;

		if spheroid_size_factor > 1.0 {
			Some(split)
		} else {
			None
		}
	}
}

#[derive(Debug, Copy, Clone)]
struct Position {
	/// Position of the chunk being carved
	chunk: GlobalColumnPosition,
	/// Absolute x position in the world
	x: f64,
	/// Absolute y position in the world
	y: f64,
	/// Absolute z position in the world
	z: f64,
	/// Horizontal angle
	yaw: f32,
	/// Vertical angle
	pitch: f32,
	/// Rate of change for the horizontal angle
	yaw_velocity: f32,
	/// Rate of change for the vertical angle
	pitch_velocity: f32,
}

impl Position {
	fn new(chunk: GlobalColumnPosition, block: (f64, f64, f64)) -> Self {
		Position::with_angles(chunk, block, 0.0, 0.0)
	}

	fn with_angles(
		chunk: GlobalColumnPosition, block: (f64, f64, f64), yaw: f32, pitch: f32,
	) -> Self {
		Position {
			chunk,
			x: block.0,
			y: block.1,
			z: block.2,
			yaw,
			pitch,
			yaw_velocity: 0.0,
			pitch_velocity: 0.0,
		}
	}

	fn step(&mut self, rng: &mut Random, pitch_keep: f32) {
		let cos_pitch = trig::cos(self.pitch);

		self.x += (trig::cos(self.yaw) * cos_pitch) as f64;
		self.y += trig::sin(self.pitch) as f64;
		self.z += (trig::sin(self.yaw) * cos_pitch) as f64;

		self.pitch *= pitch_keep;
		self.pitch += self.pitch_velocity * 0.1;
		self.yaw += self.yaw_velocity * 0.1;

		self.pitch_velocity *= 0.9;
		self.yaw_velocity *= 0.75;
		self.pitch_velocity += (rng.next_f32() - rng.next_f32()) * rng.next_f32() * 2.0;
		self.yaw_velocity += (rng.next_f32() - rng.next_f32()) * rng.next_f32() * 4.0;
	}

	fn distance_from_chunk_squared(&self) -> f64 {
		let distance_x = self.x - self.chunk.x() as f64 * 16.0 - 8.0;
		let distance_z = self.z - self.chunk.z() as f64 * 16.0 - 8.0;

		distance_x * distance_x + distance_z * distance_z
	}

	fn out_of_chunk(&self, spheroid: &SpheroidSize) -> bool {
		let horizontal_diameter = spheroid.horizontal_diameter();

		self.x < self.chunk.x() as f64 * 16.0 - 8.0 - horizontal_diameter
			|| self.z < self.chunk.z() as f64 * 16.0 - 8.0 - horizontal_diameter
			|| self.x > self.chunk.x() as f64 * 16.0 + 24.0 + horizontal_diameter
			|| self.z > self.chunk.z() as f64 * 16.0 + 24.0 + horizontal_diameter
	}

	fn spheroid(&self, size: SpheroidSize) -> Option<Spheroid> {
		let lower = (
			math::floor_clamped(self.x - size.horizontal) as i32 - self.chunk.x() * 16 - 1,
			math::floor_clamped(self.y - size.vertical) as i32 - 1,
			math::floor_clamped(self.z - size.horizontal) as i32 - self.chunk.z() * 16 - 1,
		);

		let upper = (
			math::floor_clamped(self.x + size.horizontal) as i32 - self.chunk.x() * 16 + 1,
			math::floor_clamped(self.y + size.vertical) as i32 + 1,
			math::floor_clamped(self.z + size.horizontal) as i32 - self.chunk.z() * 16 + 1,
		);

		let lower_clamped = (
			min(max(lower.0, 0), 16) as u8,
			min(max(lower.1, 1), 255) as u8,
			min(max(lower.2, 0), 16) as u8,
		);

		let upper_clamped = (
			min(max(upper.0, 0), 16) as u8,
			min(max(upper.1, 0), 120) as u8,
			min(max(upper.2, 0), 16) as u8,
		);

		if lower_clamped.0 >= upper_clamped.0
			|| lower_clamped.1 >= upper_clamped.1
			|| lower_clamped.2 >= upper_clamped.2
		{
			// Nothing to be carved
			// The spheroid is not in the chunk, but this case is not caught by out_of_chunk
			return None;
		}

		assert_ne!(lower_clamped.0, 16, "Lower limit X cannot be 16");
		assert!(lower_clamped.1 < 128, "Lower limit Y cannot be 128 or above");
		assert_ne!(lower_clamped.2, 16, "Lower limit Z cannot be 16");

		assert_ne!(upper_clamped.0, 0, "Upper limit X cannot be 0");
		assert_ne!(upper_clamped.1, 0, "Upper limit Y cannot be 0");
		assert_ne!(upper_clamped.2, 0, "Upper limit Z cannot be 0");

		Some(Spheroid {
			center: (self.x, self.y, self.z),
			size,
			lower: ColumnPosition::new(lower_clamped.0, lower_clamped.1, lower_clamped.2),
			upper: ColumnPosition::new(
				upper_clamped.0 - 1,
				upper_clamped.1 - 1,
				upper_clamped.2 - 1,
			),
		})
	}
}

#[derive(Debug)]
pub enum Outcome {
	Split,
	Constrict,
	Unreachable,
	OutOfChunk,
	Carve(Option<Spheroid>),
	Done,
}

impl Outcome {
	pub fn continues(&self) -> bool {
		match *self {
			Outcome::Split => false,
			Outcome::Constrict => true,
			Outcome::Unreachable => false,
			Outcome::OutOfChunk => true,
			Outcome::Carve(_) => true,
			Outcome::Done => false,
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct SpheroidSize {
	/// Radius on the X/Z axis
	pub horizontal: f64,
	/// Radius on the Y axis
	pub vertical: f64,
}

impl SpheroidSize {
	fn from_horizontal(horizontal: f64, vertical_multiplier: f64) -> Self {
		SpheroidSize { horizontal, vertical: horizontal * vertical_multiplier }
	}

	fn horizontal_diameter(&self) -> f64 {
		self.horizontal * 2.0
	}
}

#[derive(Debug)]
pub struct Spheroid {
	/// Center of the spheroid
	pub center: (f64, f64, f64),
	/// Size of the spheroid
	pub size: SpheroidSize,
	/// Lower bounds of the feasible region, in chunk coordinates: [0,16), [0,256), [0,16)
	pub lower: ColumnPosition,
	/// Upper bounds of the feasible region, in chunk coordinates: [0,16), [0,256), [0,16)
	/// Forms an inclusive range - these are the maximum values of the coordinates on each axis
	pub upper: ColumnPosition,
}
