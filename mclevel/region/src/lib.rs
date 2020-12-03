use std::io::{Write, Result, Seek, SeekFrom};
use std::fmt::{self, Formatter, Display, Debug};
use std::time::{SystemTime, UNIX_EPOCH};

mod zlib;

pub use zlib::{ZlibBuffer, ZlibOutput};

pub struct RegionWriter<O> where O: Write + Seek {
	header: Box<[u8; 8192]>,
	out: O,
	start: u64,
	offset_pages: u64
}

impl<O> RegionWriter<O> where O: Write + Seek {
	pub fn start(mut out: O) -> Result<Self> {
		let start = out.seek(SeekFrom::Current(0))?;
		out.write_all(&[0; 8192])?;
		
		Ok(RegionWriter {
			header: Box::new([0; 8192]),
			out,
			start,
			offset_pages: 2
		})
	}
	
	pub fn column(&mut self, x: u8, z: u8, buffer: &ZlibBuffer) -> Result<()> {
		let header = ChunkHeader {
			len: buffer.compressed_len() as u32 + 1,
			compression: 2
		};

		// compressed data len + a header of size 5
		let total_len = (buffer.compressed_len() + 5) as u32;

		let padding = header.required_padding();

		let start = self.offset_pages as u32;
		let len_pages = (total_len / 4096 + (padding != 0) as u32) as u8;

		self.offset_pages += len_pages as u64;

		RegionHeaderMut::new(&mut self.header).location(x, z, ChunkLocation::from_parts(
			start, len_pages
		));

		// Some sanity checks to make sure that we aren't writing corrupted data
		let mut written_len = 0;

		written_len += header.into_bytes().len();
		written_len += buffer.0.len();
		written_len += padding as usize;

		let pos = self.out.seek(SeekFrom::Current(0))?;
		let expected_pos = (start as u64) * 4096;

		// The position of the written chunk in the file should match what we wrote to the header
		assert_eq!(pos, expected_pos);

		// We are always writing a multiple of 4096 bytes
		assert_eq!(written_len % 4096, 0);

		// The length stored to the header should match
		assert_eq!(written_len / 4096, len_pages as usize);

		self.out.write_all(&header.into_bytes())?;
		self.out.write_all(&buffer.0)?;
		self.out.write_all(&[0; 4096][..padding as usize])
	}
	
	pub fn finish(mut self) -> Result<()> {
		self.out.seek(SeekFrom::Start(self.start))?;
		self.out.write_all(&self.header[..])?;
		self.out.seek(SeekFrom::Start(self.start + self.offset_pages * 4096))?;

		Ok(())
	}
}

// TODO: this is unused
/*pub struct RegionHeader<'a>(&'a [u8; 8192]);
impl<'a> RegionHeader<'a> {
	pub fn new(data: &'a [u8; 8192]) -> Self {
		RegionHeader(data)
	}
	
	/// Gets the location of this chunk in the file.
	/// # Panics
	/// If X or Z is greater than or equal to 32, the function will panic.
	pub fn location(&self, x: u8, z: u8) -> Option<ChunkLocation> {
		if x >= 32 || z >= 32 {
			panic!("Chunk location out of bounds in region file: {}, {}", x, z)
		}
		
		let idx = ((x as usize) | ((z as usize)<<5)) * 4;
		ChunkLocation::new(BigEndian::read_u32(&self.0[idx..]))
	}
	
	/// Gets the timestamp this chunk was saved at.
	/// # Panics
	/// If X or Z is greater than or equal to 32, the function will panic.
	pub fn timestamp(&self, x: u8, z: u8) -> ChunkTimestamp {
		if x >= 32 || z >= 32 {
			panic!("Chunk location out of bounds in region file: {}, {}", x, z)
		}
		
		let idx = ((x as usize) | ((z as usize)<<5)) * 4 + 4096;
		ChunkTimestamp::new(BigEndian::read_u32(&self.0[idx..]))
	}
}*/

pub struct RegionHeaderMut<'a>(&'a mut [u8; 8192]);
impl<'a> RegionHeaderMut<'a> {
	pub fn new(data: &'a mut [u8; 8192]) -> Self {
		RegionHeaderMut(data)
	}
	
	/// Sets the location of this chunk in the file.
	/// # Panics
	/// If X or Z is greater than or equal to 32, the function will panic.
	pub fn location(&mut self, x: u8, z: u8, location: ChunkLocation) {
		if x >= 32 || z >= 32 {
			panic!("Chunk location out of bounds in region file: {}, {}", x, z)
		}
		
		let idx = ((x as usize) | ((z as usize)<<5)) * 4;
		
		write_u32_be(&mut self.0[idx..idx+4], location.inner());
	}
	
	/// Sets the timestamp this chunk was saved at.
	/// # Panics
	/// If X or Z is greater than or equal to 32, the function will panic.
	pub fn timestamp(&mut self, x: u8, z: u8, timestamp: ChunkTimestamp) {
		if x >= 32 || z >= 32 {
			panic!("Chunk location out of bounds in region file: {}, {}", x, z)
		}
		
		let idx = ((x as usize) | ((z as usize)<<5)) * 4 + 4096;

		write_u32_be(&mut self.0[idx..idx+4], timestamp.into_unix_seconds());
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ChunkLocation(u32);
impl ChunkLocation {
	pub fn from_parts(offset: u32, len: u8) -> Self {
		ChunkLocation((offset << 8) | (len as u32))
	}
	
	pub fn new(loc: u32) -> Option<Self> {
		if loc == 0 {
			None
		} else {
			Some(ChunkLocation(loc))
		}
	}
	
	/// Returns the contained raw value, which is guaranteed to be non-zero.
	pub fn inner(&self) -> u32 {
		self.0
	}
	
	/// Returns the offset in pages (4096 bytes) of this chunk from the start of the file.
	pub fn offset(&self) -> u32 {
		self.0 >> 8
	}
	
	/// Returns the offset in bytes of this chunk from the start of the file.
	pub fn offset_bytes(&self) -> u64 {
		(self.offset() as u64) * 4096
	}

	/// Returns the size of the chunk in pages (4096 bytes).
	pub fn len(&self) -> u8 {
		(self.0 & 0xFF) as u8
	}
	
	/// Returns the size of the chunk in bytes.
	pub fn len_bytes(&self) -> u32 {
		(self.0 & 0xFF) * 4096
	}
	
	pub fn end(&self) -> u32 {
		self.offset() + (self.len() as u32)
	}
	
	pub fn end_bytes(&self) -> u32 {
		self.end() * 4096
	}
}

impl Display for ChunkLocation {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "at {}, len {} pages", self.offset(), self.len())
	}
}

impl Debug for ChunkLocation {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "ChunkLocation {{ offset: {}, len: {} }}", self.offset(), self.len())
	}
}

/// Unix time in seconds when the chunk was last saved.
/// Susceptible to the Year 2038 problem, and relatively useless.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ChunkTimestamp(u32);
impl ChunkTimestamp {
	pub fn from_unix_seconds(seconds: u32) -> Self {
		ChunkTimestamp(seconds)
	}
	
	pub fn now() -> Option<Self> {
		let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
		
		if seconds < u32::max_value() as u64 {
			Some(ChunkTimestamp(seconds as u32))
		} else {
			None
		}
	}
	
	pub fn into_unix_seconds(self) -> u32 {
		self.0
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ChunkHeader {
	pub len: u32, 
	pub compression: u8
}

impl ChunkHeader {
	pub fn from_bytes(bytes: [u8; 5]) -> Self {
		ChunkHeader {
			len: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
			compression: bytes[4]
		}
	}

	pub fn required_padding(self) -> u32 {
		4096 - (self.len + 4) % 4096
	}
	
	pub fn into_bytes(self) -> [u8; 5] {
		let mut bytes = [0u8; 5];

		write_u32_be(&mut bytes[0..4], self.len);
		bytes[4] = self.compression;

		bytes
	}
}

fn write_u32_be(slice: &mut [u8], value: u32) {
	assert_eq!(slice.len(), 4);

	let bytes = u32::to_be_bytes(value);

	slice[0] = bytes[0];
	slice[1] = bytes[1];
	slice[2] = bytes[2];
	slice[3] = bytes[3];
}
