include!(concat!(env!("OUT_DIR"), "/sin_table.rs"));

pub fn sin(f: f32) -> f32 {
	sin_index(((f * 10430.38) as i32) as u16)
}

pub fn cos(f: f32) -> f32 {
	sin_index(((f * 10430.38 + 16384.0) as i32) as u16)
}

/// Computes the sin value for an index, where an
/// index of 65536 is equal to 2π, and an index of 0 is 0.
fn sin_index(index: u16) -> f32 {
	// A special case...
	if index == 32768 {
		// *almost* zero... it's 0.00000000000000012246469

		return f32::from_bits(0x250D3132);
	}

	// Trigonometric identity: sin(-x) = -sin(x)
	// Given a domain of 0 ≤ x < 2π, just negate the value if x > π
	// This allows the sin table size to be halved.
	// (x ^ (1 << 31)) = -x
	let neg = ((index & 0x8000) as u32) << 16;

	f32::from_bits(sin_index_half(index & 0x7FFF) ^ neg)
}

/// Computes the sin value on the range of 0 ≤ x < π
fn sin_index_half(mut index: u16) -> u32 {
	// 1 if π/2 ≤ x, 0 otherwise
	let invert = (index & 0x4000) >> 14;

	// If invert is 1, then results in 0xFFFF, otherwise results in 0
	let full_invert = 0u16.wrapping_sub(invert);

	// 0x8001 if π/2 ≤ x, 0 otherwise
	let sub_from = (invert << 15) + invert;
	
	// Trigonometric identity: sin(x) = sin(π/2 - x)
	// Computes 0x8000 - index if x > π, doesn't change index otherwise.
	index = sub_from.wrapping_add(index ^ full_invert);

	// Special case: an index of 16384 has the same value as 16383
	index = clamp_index(index);

	SIN_TABLE[index as usize]
}

/// Clamps the input index to the range of 0 to 16383.
fn clamp_index(a: u16) -> u16 {
	let difference = a.wrapping_sub(16383);
	let extended_sign = ((difference as i16) >> 15) as u16;

	u16::wrapping_add(16383, difference & extended_sign)
}

#[cfg(test)]
mod test {
	#[test]
	fn test_sin() {
		let java = crate::test::read_u32s("JavaSinTable");

		assert_eq!(java.len(), 65536);

		for (index, &j) in java.iter().enumerate() {
			let r = super::sin_index(index as u16).to_bits();

			if r != j {
				panic!("trig::test_sin: mismatch @ index {}: {} (R) != {} (J)", index, r, j);
			}
		}
	}
}
