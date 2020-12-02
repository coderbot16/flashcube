mod chunk;
mod layer;
mod bulk;

pub use self::chunk::ChunkNibbles;
pub use self::layer::LayerNibbles;
pub use self::bulk::BulkNibbles;

use crate::component::Component;

/// Returns the chunk_yzx index into a nibble array. Returns in the form (index, shift).
fn nibble_index(index: usize) -> (usize, u8) {
	(index >> 1, ((index & 1) as u8) << 2)
}

impl Component for u4 {
	type Chunk = ChunkNibbles;
	type Layer = LayerNibbles;
	type Bulk = ();
}

/// The 4-bit unsigned integer type.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct u4(u8);
impl u4 {
	/// Casts a u8 to a u4, truncating the value in the process.
	#[inline]
	pub fn new(x: u8) -> Self {
		u4(x & 0xF)
	}

	/// Casts a u4 to an u8, returning a value in the range 0-15 (inclusive).
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::u4;
	///
	/// assert_eq!(u4::new(15).raw(), 15u8);
	/// ```
	#[inline]
	pub fn raw(self) -> u8 {
		self.0
	}

	/// Adds a u4 to another u4, capping the result to 15 if it were to overflow.
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::u4;
	///
	/// assert_eq!(u4::new(15).saturating_add(u4::new(7)), u4::new(15));
	/// assert_eq!(u4::new(7).saturating_add(u4::new(3)), u4::new(10));
	/// ```
	#[inline]
	pub fn saturating_add(self, rhs: Self) -> Self {
		u4(::std::cmp::min(self.0 + rhs.0, 15))
	}

	/// Subtracts a u4 from another u4, capping the result to 0 if it were to underflow.
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::u4;
	///
	/// assert_eq!(u4::new(15).saturating_sub(u4::new(7)), u4::new(8));
	/// assert_eq!(u4::new(7).saturating_sub(u4::new(3)), u4::new(4));
	/// ```
	#[inline]
	pub fn saturating_sub(self, rhs: Self) -> Self {
		u4(self.0.saturating_sub(rhs.0))
	}
}

/// A vector of 2 u4s. This is implemented as a single u8, but emulates operations on true SIMD types.
/// One thing to note is that no operations on this type take a mutable reference. This means that
/// the following situation is completely valid:
///
/// ```
/// # use vocs::nibbles::{u4, u4x2};
///
/// let mut pair = u4x2::new(u4::new(1), u4::new(4));
///
/// pair.replace(false as u8, u4::new(2)); // creates a new u4x2, then throws it away
///
/// assert_eq!(pair, u4x2::new(u4::new(1), u4::new(4)));
/// ```
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Default)]
pub struct u4x2(u8);
impl u4x2 {
	/// Converts a value represented as `(b << 4 | a)` to a `u4x2`.
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::{u4, u4x2};
	/// 
	/// let pair = u4x2::from_ba(15*16 + 5 /*or, 245*/);
	/// assert_eq!(pair.a(), u4::new(5));
	/// assert_eq!(pair.b(), u4::new(15));
	/// ```
	#[inline]
	pub fn from_ba(ba: u8) -> Self {
		u4x2(ba)
	}

	/// Creates a `u4x2` where both elements have the same value.
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::{u4, u4x2};
	/// 
	/// let pair = u4x2::splat(u4::new(1));
	/// assert_eq!(pair.a(), u4::new(1));
	/// assert_eq!(pair.b(), u4::new(1));
	/// ```
	#[inline]
	pub fn splat(v: u4) -> Self {
		u4x2((v.0 << 4) | v.0)
	}

	/// Creates a `u4x2` from 2 different elements.
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::{u4, u4x2};
	/// 
	/// let pair = u4x2::new(u4::new(1), u4::new(4));
	/// assert_eq!(pair.a(), u4::new(1));
	/// assert_eq!(pair.b(), u4::new(4));
	/// ```
	#[inline]
	pub fn new(a: u4, b: u4) -> Self {
		u4x2(a.0 | (b.0 << 4))
	}

	/// Copies a single element out of the `u4x2`
	/// The discriminator (`d`) will select `b` if it is nonzero, `a` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::{u4, u4x2};
	/// 
	/// assert_eq!(u4x2::new(u4::new(1), u4::new(4)).extract(false as u8), u4x2::new(u4::new(1), u4::new(4)).a());
	/// assert_eq!(u4x2::new(u4::new(1), u4::new(4)).extract(true as u8), u4x2::new(u4::new(1), u4::new(4)).b());
	/// assert_eq!(u4x2::new(u4::new(1), u4::new(4)).extract(false as u8), u4::new(1));
	/// assert_eq!(u4x2::new(u4::new(1), u4::new(4)).extract(true as u8), u4::new(4));
	/// ```
	#[inline]
	pub fn extract(self, d: u8) -> u4 {
		let shift = ((d != 0) as u8)  * 4;
		let single = self.0 & (0xF << shift);

		u4(single >> shift)
	}

	/// Sets a single element of the `u4x2` to zero.
	/// The discriminator (`d`) will select `b` if it is nonzero, `a` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::{u4, u4x2};
	/// 
	/// assert_eq!(u4x2::new(u4::new(1), u4::new(4)).clear(false as u8), u4x2::new(u4::new(0), u4::new(4)));
	/// assert_eq!(u4x2::new(u4::new(0), u4::new(4)).clear(false as u8), u4x2::new(u4::new(1), u4::new(4)).replace(false as u8, u4::new(0)));
	/// ```
	#[inline]
	pub fn clear(self, d: u8) -> Self {
		let shift = ((d != 0) as u8)  * 4;

		u4x2(!((!self.0) | (0xF << shift)))
	}

	/// Replaces a single element of the `u4x2` with the specified value.
	/// This is equivalent to a `clear` operation then a `replace_or` operation.
	/// The discriminator (`d`) will select `b` if it is nonzero, `a` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::{u4, u4x2};
	/// 
	/// assert_eq!(u4x2::new(u4::new(1), u4::new(4)).replace(false as u8, u4::new(2)), u4x2::new(u4::new(2), u4::new(4)));
	/// ```
	#[inline]
	pub fn replace(self, d: u8, v: u4) -> Self {
		let shift = ((d != 0) as u8)  * 4;

		let cleared = !((!self.0) | (0xF << shift));

		u4x2(cleared | (v.0 << shift))
	}

	/// Replaces a single element of the `u4x2` with the specified value with bitwise or.
	/// The previous value is not cleared, meaning that the resulting value is a combination
	/// of the two values.
	/// The discriminator (`d`) will select `b` if it is nonzero, `a` otherwise.
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::{u4, u4x2};
	/// 
	/// assert_eq!(u4x2::new(u4::new(1), u4::new(4)).replace_or(false as u8, u4::new(2)), u4x2::new(u4::new(3), u4::new(4)));
	/// ```
	#[inline]
	pub fn replace_or(self, d: u8, v: u4) -> Self {
		let shift = ((d != 0) as u8)  * 4;

		u4x2(self.0 | (v.0 << shift))
	}

	/// Extracts the `a` element from this `u4x2`.
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::{u4, u4x2};
	/// 
	/// assert_eq!(u4x2::new(u4::new(3), u4::new(4)).a(), u4::new(3));
	/// ```
	#[inline]
	pub fn a(self) -> u4 {
		u4(self.0 & 0xF)
	}

	/// Extracts the `b` element from this `u4x2`.
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::{u4, u4x2};
	/// 
	/// assert_eq!(u4x2::new(u4::new(3), u4::new(4)).b(), u4::new(4));
	/// ```
	#[inline]
	pub fn b(self) -> u4 {
		u4(self.0 >> 4)
	}

	/// Returns a value represented as `(b << 4 | a)`.
	/// This is the counterpart to `u4x2::from_ba`.
	///
	/// # Examples
	///
	/// ```
	/// # use vocs::nibbles::u4x2;
	/// 
	/// assert_eq!(u4x2::from_ba(242).ba(), 242);
	/// ```
	#[inline]
	pub fn ba(self) -> u8 {
		self.0
	}
}