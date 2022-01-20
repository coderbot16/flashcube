/// Represents 16 nibbles as a 64-bit value

#[derive(Copy, Clone)]
pub struct u4x16(pub u64);

impl u4x16 {
	/// Computes self < other and returns the result as a mask.
	/// For each element, the mask is 1111 if self[e] < other[e],
	/// and is 0000 if self[e] >= other[e].
	pub fn lt(self, other: u4x16) -> u4x16 {
		fn lt_half(mut a: u64, mut b: u64) -> u64 {
			const COMPONENT_MASK: u64 = 0x0f0f_0f0f_0f0f_0f0f;
			const BORROW_GUARD:   u64 = 0x2020_2020_2020_2020;
			const CARRY_MASK:     u64 = 0x1010_1010_1010_1010;

			a &= COMPONENT_MASK;
			b &= COMPONENT_MASK;

			// The borrow guard ensures that any carries in the subtraction
			// are 
			let mut carry = (a | BORROW_GUARD) - b;
			carry &= CARRY_MASK;

			// Create the mask in the lower 4 bits of each 8 bit part, from
			// the carry bit in the upper 4 bits.

			// faster version of:
			// (carry >> 1) | (carry >> 2) | (carry >> 3) | (carry >> 4)
			// saves one shift and one or
			let carry2 = (carry >> 1) | (carry >> 2);
			return carry2 | (carry2 >> 2);
		}

		return u4x16(lt_half(self.0, other.0) | (lt_half(self.0 >> 4, other.0 >> 4) << 4));
	}

	/// Inputs: Two u4x16 words, self and other, as well as a u4x16 mask.
	/// In a valid mask, each 4-bit element is either 1111 or 0000.
	pub fn select(self, other: u4x16, mask: u4x16) -> u4x16 {
		return u4x16(self.0 ^ ((self.0 ^ other.0) & mask.0));
	}

	// This function uses only 24 64-bit integer operations to
	// compute the maximum values for 16 nibble values,
	// an average of 1.5 instructions per nibble, assuming
	// that the compiler optimizes well.
	//
	// - 7x and
	// - 8x or
	// - 5x lsl
	// - 1x lsr
	// - 2x xor
	// - 1x sub
	pub fn max(self, other: u4x16) -> u4x16 {
		return select(self, other, lt(self, other));
	}
}
