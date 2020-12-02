pub struct ClassicWorld {
	pub name: String,
	pub uuid: [u8; 16],
	pub time_created: i64,
	pub last_accessed: i64,
	pub last_modified: i64,
	pub spawn: (i16, i16, i16),
	pub blocks: BlockVolume
}

pub struct BlockVolume {
	pub blocks: Box<[u8]>,
	pub x_size: usize,
	pub y_size: usize,
	pub z_size: usize
}

impl BlockVolume {
	pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
		assert!(x < self.x_size);
		assert!(y < self.y_size);
		assert!(z < self.z_size);

		(y * self.z_size + z) * self.x_size + x
	}
}
