pub enum Kind {
	End,
	I8,
	I16,
	I32,
	I64,
	F32,
	F64,
	U8Array,
	String,
	List,
	Compound,
	U32Array,
	I64Array
}

pub trait Output {
	fn push(&mut self, value: u8);
	fn extend_from_slice(&mut self, slice: &[u8]);
}

impl<T> Output for &mut T where T: Output {
	fn push(&mut self, value: u8) {
		T::push(self, value)
	}

	fn extend_from_slice(&mut self, slice: &[u8]) {
		T::extend_from_slice(self, slice);
	}
}

impl Output for Vec<u8> {
	fn push(&mut self, value: u8) {
		Vec::push(self, value)
	}

	fn extend_from_slice(&mut self, slice: &[u8]) {
		Vec::extend_from_slice(self, slice)
	}
}

pub struct CompoundWriter<T: Output> {
	out: T
}

impl<T: Output> CompoundWriter<T> {
	/// Begins a new compound tag.
	pub fn start(name: &str, out: T) -> Self {
		let mut writer = CompoundWriter { out };

		writer.header(Kind::Compound, name);
		writer
	}

	pub fn write<F>(name: &str, out: T, filler: F) -> T where F: FnOnce(&mut CompoundWriter<T>) {
		let mut writer = CompoundWriter::start(name, out);

		filler(&mut writer);

		writer.end()
	}

	/// Ends the compound tag, returning the buffer.
	pub fn end(mut self) -> T {
		self.out.push(0);
		self.out
	}

	fn header(&mut self, kind: Kind, name: &str) {
		self.out.push(kind as u8);

		assert!(name.len() <= 32767, "Tag name too long: {}", name);
		self.out.extend_from_slice(&u16::to_be_bytes(name.len() as u16));
		self.out.extend_from_slice(name.as_bytes());
	}

	pub fn bool(&mut self, name: &str, value: bool) -> &mut Self {
		self.i8(name, value as i8)
	}

	pub fn i8(&mut self, name: &str, value: i8) -> &mut Self {
		self.header(Kind::I8, name);
		self.out.push(value as u8);

		self
	}

	pub fn i16(&mut self, name: &str, value: i16) -> &mut Self {
		self.header(Kind::I16, name);
		self.out.extend_from_slice(&value.to_be_bytes());

		self
	}

	pub fn i32(&mut self, name: &str, value: i32) -> &mut Self {
		self.header(Kind::I32, name);
		self.out.extend_from_slice(&value.to_be_bytes());

		self
	}

	pub fn i64(&mut self, name: &str, value: i64) -> &mut Self {
		self.header(Kind::I64, name);
		self.out.extend_from_slice(&value.to_be_bytes());

		self
	}

	pub fn f32(&mut self, name: &str, value: f32) -> &mut Self {
		self.header(Kind::F32, name);
		self.out.extend_from_slice(&value.to_bits().to_be_bytes());

		self
	}

	pub fn f64(&mut self, name: &str, value: f64) -> &mut Self {
		self.header(Kind::F64, name);
		self.out.extend_from_slice(&value.to_bits().to_be_bytes());

		self
	}

	pub fn string(&mut self, name: &str, value: &str) -> &mut Self {
		self.header(Kind::String, name);

		assert!(value.len() <= 32767, "Tag string value too long: {}", value);
		self.out.extend_from_slice(&u16::to_be_bytes(value.len() as u16));
		self.out.extend_from_slice(value.as_bytes());

		self
	}

	fn array_length(&mut self, len: usize) {
		assert!(len <= std::i32::MAX as usize, "Array too long: {} (maximum length: {})", len, std::i32::MAX);

		self.out.extend_from_slice(&u32::to_be_bytes(len as u32));
	}

	pub fn u8_array(&mut self, name: &str, value: &[u8]) -> &mut Self {
		self.header(Kind::U8Array, name);
		self.array_length(value.len());

		self.out.extend_from_slice(value);

		self
	}

	pub fn u32_array(&mut self, name: &str, value: &[u32]) -> &mut Self {
		self.header(Kind::U32Array, name);
		self.array_length(value.len());

		for &entry in value {
			self.out.extend_from_slice(&entry.to_be_bytes());
		}

		self
	}

	pub fn compound_writer(&mut self, name: &str) -> CompoundWriter<&mut T> {
		CompoundWriter::start(name, &mut self.out)
	}

	pub fn compound<F>(&mut self, name: &str, filler: F) where F: FnOnce(&mut CompoundWriter<&mut T>) {
		let mut writer = CompoundWriter::start(name, &mut self.out);

		filler(&mut writer);

		writer.end();
	}

	// TODO: List
	pub fn compound_array<F>(&mut self, name: &str, len: usize, filler: F) -> &mut Self where F: FnOnce(&mut CompoundArrayWriter<T>) {
		self.header(Kind::List, name);

		if len == 0 {
			// Kind::End + length of 0
			self.out.extend_from_slice(&[0; 5]);

			return self;
		}

		self.out.push(Kind::Compound as u8);
		self.array_length(len);

		let mut writer = CompoundArrayWriter {
			out: &mut self.out,
			remaining: len
		};

		filler(&mut writer);

		assert_eq!(writer.remaining, 0, "Attempted to end an incomplete CompoundArrayWriter, {} element(s) remaining", writer.remaining);

		self
	}
}

pub struct CompoundArrayWriter<'w, T: Output> {
	out: &'w mut T,
	remaining: usize
}

impl<'w, T> CompoundArrayWriter<'w, T> where T: Output {
	pub fn compound<F>(&mut self, filler: F) where F: FnOnce(&mut CompoundWriter<&mut T>) {
		assert_ne!(self.remaining, 0);
		self.remaining -= 1;

		let mut writer = CompoundWriter {
			// re-borrow the output
			out: &mut *self.out
		};

		filler(&mut writer);

		writer.end();
	}
}
