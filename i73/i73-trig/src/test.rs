use std::fs::File;
use std::io::Read;

#[test]
fn test_sin() {
	let java = read_u32s("JavaSinTable");

	assert_eq!(java.len(), 65536);

	for (index, &j) in java.iter().enumerate() {
		let r = crate::sin_index(index as u16).to_bits();

		if r != j {
			panic!("trig::test_sin: mismatch @ index {}: {} (R) != {} (J)", index, r, j);
		}
	}
}

pub fn read_u32s(name: &str) -> Vec<u32> {
	let file = File::open(format!("test_data/{}.txt", name)).unwrap();
	let mut data = Vec::new();

	let mut term = 0u32;
	for byte in file.bytes() {
		let c = (byte.unwrap() as char).to_lowercase().next().unwrap();

		if c.is_whitespace() {
			data.push(term);

			term = 0;
		} else {
			term <<= 4;
			term |= if c >= '0' && c <= '9' {
				(c as u32) - ('0' as u32)
			} else if c >= 'a' && c <= 'f' {
				(c as u32) - ('a' as u32) + 10
			} else {
				panic!("Bad hex character {}", c);
			};
		}
	}

	data.push(term);

	data
}
