mod chunk;
mod layer;
mod bulk;

pub use self::chunk::ChunkNibbles;
pub use self::layer::LayerNibbles;
pub use self::bulk::BulkNibbles;

use component::Component;

/// Returns the chunk_yzx index into a nibble array. Returns in the form (index, shift).
fn nibble_index(index: usize) -> (usize, u8) {
	(index >> 1, ((index & 1) as u8) << 2)
}

impl Component for u4 {
	type Chunk = ChunkNibbles;
	type Layer = LayerNibbles;
	type Bulk = ();
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct u4(u8);
impl u4 {
	pub fn new(x: u8) -> Self {
		u4(x & 0xF)
	}

	pub fn raw(self) -> u8 {
		self.0
	}

	pub fn saturating_add(self, b: Self) -> Self {
		u4(::std::cmp::min(self.0 + b.0, 15))
	}

	pub fn saturating_sub(self, b: Self) -> Self {
		u4(self.0.saturating_sub(b.0))
	}
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Default)]
pub struct u4x2(u8);
impl u4x2 {
	pub fn from_ba(ba: u8) -> Self {
		u4x2(ba)
	}

	pub fn splat(v: u4) -> Self {
		u4x2((v.0 << 4) | v.0)
	}

	pub fn new(a: u4, b: u4) -> Self {
		u4x2(a.0 | (b.0 << 4))
	}

	pub fn extract(self, d: u8) -> u4 {
		let shift = (d&1) * 4;
		let single = self.0 & (0xF << shift);

		u4(single >> shift)
	}

	pub fn clear(self, d: u8) -> Self {
		let shift = (d&1) * 4;

		u4x2(!((!self.0) | (0xF << shift)))
	}

	pub fn replace(self, d: u8, v: u4) -> Self {
		let shift = (d&1) * 4;

		let cleared = !((!self.0) | (0xF << shift));

		u4x2(cleared | (v.0 << shift))
	}

	pub fn replace_or(self, d: u8, v: u4) -> Self {
		let shift = (d&1) * 4;

		u4x2(self.0 | (v.0 << shift))
	}

	pub fn a(self) -> u4 {
		u4(self.0 & 0xF)
	}

	pub fn b(self) -> u4 {
		u4(self.0 >> 4)
	}

	pub fn ba(self) -> u8 {
		self.0
	}
}