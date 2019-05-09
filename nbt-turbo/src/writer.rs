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
	I32Array,
	I64Array
}

pub trait Output {
	fn push(&mut self, value: u8);
	fn extend_from_slice(&mut self, slice: &[u8]);
}

impl<T> Output for T where T: AsMut<Vec<u8>> {
	fn push(&mut self, value: u8) {
		self.as_mut().push(value)
	}
	fn extend_from_slice(&mut self, slice: &[u8]) {
		self.as_mut().extend_from_slice(slice);
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

	/// Ends the compound tag, returning the buffer.
	pub fn end(mut self) -> T {
		self.out.push(0);
		self.out
	}

	/// Same as end, but for child compounds.
	pub fn close(&mut self) {
		self.out.push(0);
	}

	fn header(&mut self, kind: Kind, name: &str) {
		self.out.push(kind as u8);

		assert!(name.len() <= 32767, "Tag name too long: {}", name);
		self.out.extend_from_slice(&u16::to_be_bytes(name.len() as u16));
		self.out.extend_from_slice(name.as_bytes());
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

	pub fn u8_array(&mut self, name: &str, value: &[u8]) -> &mut Self {
		self.header(Kind::U8Array, name);

		assert!(value.len() <= std::i32::MAX as usize, "Byte array too long: {} (maximum length: {})", value.len(), std::i32::MAX);
		self.out.extend_from_slice(&u32::to_be_bytes(value.len() as u32));
		self.out.extend_from_slice(value);

		self
	}

	pub fn string(&mut self, name: &str, value: &str) -> &mut Self {
		self.header(Kind::String, name);

		assert!(value.len() <= 32767, "Tag string value too long: {}", value);
		self.out.extend_from_slice(&u16::to_be_bytes(value.len() as u16));
		self.out.extend_from_slice(value.as_bytes());

		self
	}

	// TODO: List
}

impl<T> CompoundWriter<T> where T: Output + AsMut<Vec<u8>> {
	pub fn compound(&mut self, name: &str) -> CompoundWriter<&mut T> {
		CompoundWriter::start(name, &mut self.out)
	}
}