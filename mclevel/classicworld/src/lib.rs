pub struct ClassicWorld {
	name: String,
	uuid: [u8; 16],
	time_created: i64,
	last_accessed: i64,
	last_modified: i64,
	spawn: (i16, i16, i16),
	blocks: BlockVolume
}

pub struct BlockVolume {
	blocks: Box<[u8]>,
	x_size: usize,
	y_size: usize,
	z_size: usize
}

impl BlockVolume {
	fn index(&self, x: usize, y: usize, z: usize) -> usize {
		assert!(x < self.x_size);
		assert!(y < self.y_size);
		assert!(z < self.z_size);

		(y * self.z_size + z) * self.x_size + x
	}
}
