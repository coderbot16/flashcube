use super::{u4, u4x2, nibble_index};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BulkNibbles {
	/// Internal pairs.
	data: Vec<u4x2>,
	/// Whether the last element is valid.
	last: bool
}

impl BulkNibbles {
	pub fn new(len: usize) -> Self {
		let (data_len, shift) = nibble_index(len);
		let last = shift != 0;

		BulkNibbles {
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
		self.data.len() * 2 + self.last as usize
	}
}