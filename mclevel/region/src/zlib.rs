use deflate::Compression;
use deflate::write::ZlibEncoder;
use nbt_turbo::writer::Output;
use std::io::Write;

pub struct ZlibBuffer(pub(crate) Vec<u8>);

impl ZlibBuffer {
	pub fn compressed_len(&self) -> usize {
		self.0.len()
	}
}

pub struct ZlibOutput {
	buffer: Vec<u8>,
	writer: ZlibEncoder<Vec<u8>>
}

impl ZlibOutput {
	pub fn new() -> Self {
		Self::with_capacity(4096)
	}

	pub fn with_capacity(capacity: usize) -> Self {
		ZlibOutput {
			buffer: Vec::with_capacity(256),
			writer: ZlibEncoder::new(Vec::with_capacity(capacity), Compression::Default)
		}
	}

	fn flush(&mut self) {
		self.writer.write_all(&self.buffer).unwrap();
		self.buffer.clear();
	}

	fn maybe_flush(&mut self) {
		if self.buffer.len() > 255 {
			self.flush();
		}
	}

	pub fn finish(mut self) -> ZlibBuffer {
		self.flush();

		ZlibBuffer(self.writer.finish().unwrap())
	}
}

impl Output for ZlibOutput {
	fn push(&mut self, byte: u8) {
		self.buffer.push(byte);
		self.maybe_flush();
	}

	fn extend_from_slice(&mut self, slice: &[u8]) {
		if slice.len() < 128 {
			self.buffer.extend_from_slice(slice);
			self.maybe_flush();
		} else {
			self.flush();
			self.writer.write_all(slice).unwrap();
		}
	}
}
