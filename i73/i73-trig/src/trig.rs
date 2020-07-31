use std::cmp;

include!(concat!(env!("OUT_DIR"), "/sin_table.rs"));

pub fn sin(f: f32) -> f32 {
	sin_index(((f * 10430.38) as i32) as u32)
}

pub fn cos(f: f32) -> f32 {
	sin_index(((f * 10430.38 + 16384.0) as i32) as u32)
}

fn sin_index(mut index: u32) -> f32 {
	// Clamp the input to the range of 0 ≤ x < 2π
	index &= 0xFFFF;

	// A special case...
	if index == 32768 {
		// *almost* zero... it's 0.00000000000000012246469

		return f32::from_bits(0x250D3132);
	}

	// Trigonometric identity: sin(-x) = -sin(x)
	// Given a domain of 0 ≤ x < 2π, just negate the value if x > π
	// This allows the sin table size to be halved.
	let neg = (index & 0x8000) << 16;

	// Use the half-range index
	let idx = index & 0x7FFF;
	let invert = (idx & 0x4000) >> 14;

	let full_invert = 0u32.wrapping_sub(invert);
	let sub_from = (invert << 15) + invert;
	let idx3 = cmp::min(sub_from.wrapping_add(idx ^ full_invert), 16383);

	let raw = SIN_TABLE[idx3 as usize] ^ neg;

	f32::from_bits(raw)
}

#[cfg(test)]
mod test {
	#[test]
	fn test_sin() {
		let java = crate::test::read_u32s("JavaSinTable");

		assert_eq!(java.len(), 65536);

		for (index, &j) in java.iter().enumerate() {
			let r = super::sin_index(index as u32).to_bits();

			if r != j {
				panic!("trig::test_sin: mismatch @ index {}: {} (R) != {} (J)", index, r, j);
			}
		}
	}
}
