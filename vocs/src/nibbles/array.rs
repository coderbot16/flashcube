use super::{u4, u4x2, nibble_index};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NibbleArray {
	/// Internal pairs.
	data: Vec<u4x2>,
	/// Whether the last element is valid.
	last: bool
}

impl NibbleArray {
	pub fn new(len: usize) -> Self {
		// If the length is odd, we don't have a valid last element
		let last = (len & 1) == 0;

		// Make sure that the backing array is large enough to accommodate each element when the
		// length is odd
		let data_len = (len + 1) / 2;

		NibbleArray {
			data: vec![u4x2::default(); data_len],
			last
		}
	}

	pub fn get(&self, index: usize) -> u4 {
		let len = self.len();

		if index >= len {
			panic!("index out of bounds: the len is {} but the index is {}", len, index);
		}

		let (index, shift) = nibble_index(index);

		self.data[index].extract((shift != 0) as u8)
	}

	pub fn set(&mut self, index: usize, value: u4) {
		let len = self.len();

		if index >= len {
			panic!("index out of bounds: the len is {} but the index is {}", len, index);
		}

		let (index, shift) = nibble_index(index);

		self.data[index] = self.data[index].replace((shift != 0) as u8, value);
	}

	pub fn set_or(&mut self, index: usize, value: u4) {
		let len = self.len();

		if index >= len {
			panic!("index out of bounds: the len is {} but the index is {}", len, index);
		}

		let (index, shift) = nibble_index(index);

		self.data[index] = self.data[index].replace_or((shift != 0) as u8, value);
	}

	pub fn len(&self) -> usize {
		let base_len = self.data.len() * 2;
		
		if self.last{
			base_len
		} else {
			base_len - 1
		}
	}

	pub fn iter<'a>(&'a self) -> Iter<'a> {
		Iter {
			next: &self.data,
			partial: None,
			last: self.last
		}
	}
}

pub struct Iter<'a> {
	next: &'a [u4x2],
	partial: Option<u4>,
	last: bool
}

impl<'a> Iterator for Iter<'a> {
	type Item = u4;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(partial) = self.partial.take() {
			return Some(partial)
		}

		if self.next.is_empty() {
			return None
		}

		let (first, rest) = self.next.split_at(1);
		self.next = rest;

		let joined = first[0];

		if !self.next.is_empty() || self.last {
			self.partial = Some(joined.b());
		}

		Some(joined.a())
	}
}

#[cfg(test)]
mod test {
	use crate::nibbles::u4;
	use super::NibbleArray;

	#[test]
	fn test_odd_length() {
		let mut nibbles = NibbleArray::new(1);
		let test_value = u4::new(4);

		assert_eq!(nibbles.len(), 1);

		nibbles.set(0, test_value);

		assert_eq!(nibbles.get(0), test_value);
	}

	/// Basic tests of whether we can set some sequential values in the nibble array and then read
	/// them back properly
	#[test]
	fn test_set_get_iter() {
		/// A simple function to map an index in the range of 0..32 to a u4 value
		fn value_for_index(index: usize) -> u4 {
			if index < 16 {
				u4::new(index as u8)
			} else {
				u4::new(32 - (index as u8))
			}
		}

		for len in 0..32 {
			let mut nibbles = NibbleArray::new(len);

			for index in 0..len {
				nibbles.set(index, value_for_index(index));
			}

			assert_eq!(nibbles.len(), len);

			let mut iterations = 0;

			for (index, value) in nibbles.iter().enumerate() {
				let expected_value = value_for_index(index);

				assert_eq!(expected_value, value);
				assert_eq!(expected_value, nibbles.get(index));
				iterations += 1;
			}

			assert_eq!(iterations, len);
		}
	}
}
