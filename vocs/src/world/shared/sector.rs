use spin::RwLock;
use std::ops::Index;
use position::{LayerPosition, ChunkPosition};
use std::slice;
use world::shared::{Packed, Guard};

// TODO: Design: Should locks panic by default? This would make sense with the scheduler design, but could be a similar footgun to RefCell.

pub struct SharedSector<T> where T: Packed {
	chunks: Box<[RwLock<Option<T>>]>
}

impl<T> SharedSector<T> where T: Packed {
	pub fn new() -> Self {
		let mut chunks = Vec::with_capacity(4096);

		for _ in 0..4096 {
			chunks.push(RwLock::new(None));
		}

		SharedSector { chunks: chunks.into_boxed_slice() }
	}

	pub fn set(&self, position: ChunkPosition, chunk: T) {
		*self[position].write() = Some(chunk);
	}

	// TODO: pop_first

	pub fn set_column(&self, position: LayerPosition, column: [T; 16]) {
		// TODO: This is hackish, and needs a heap allocation. Find a better way!
		// Or, wait for slice patterns.

		let mut chunks = (Box::new(column) as Box<[_]>).into_vec();

		for (index, chunk) in chunks.drain(..).enumerate() {
			let position = ChunkPosition::from_layer(index as u8, position);

			self.set(position, chunk);
		}
	}

	pub fn remove(&self, position: ChunkPosition) -> Option<T> {
		self[position].write().take()
	}

	pub fn get(&self, position: ChunkPosition) -> Option<Guard<T>> {
		let mut slot = self[position].write();
		let packed = match slot.take() {
			Some(packed) => packed,
			None => return None
		};

		Some(Guard { slot, value: Some(packed.unpack()) })
	}

	pub fn iter(&self) -> slice::Iter<RwLock<Option<T>>> {
		self.chunks.iter()
	}
}

impl<T> SharedSector<T> where T: Packed + Default {
	pub fn get_or_create(&self, position: ChunkPosition) -> Guard<T> {
		let mut slot = self[position].write();
		let packed = slot.take().unwrap_or_else(T::default);

		Guard { slot, value: Some(packed.unpack()) }
	}
}

/* TODO: Reimplement
impl<B> SharedSector<ChunkIndexed<B>> where B: Target {
	pub fn set_block_immediate(&mut self, x: u8, y: u8, z: u8, target: &B) -> Option<()> {
		let (chunk, block) = (
			ChunkPosition::new(x / 16, y / 16, z / 16),
			ChunkPosition::new(x % 16, y % 16, z % 16)
		);

		self.get_mut(chunk).map(|chunk| chunk.set_immediate(block, &target))
	}

	pub fn get_block(&self, x: u8, y: u8, z: u8) -> Option<&B> {
		let (chunk, block) = (
			ChunkPosition::new(x / 16, y / 16, z / 16),
			ChunkPosition::new(x % 16, y % 16, z % 16)
		);

		self[chunk].as_ref().map(|chunk| chunk.get(block))
	}
}*/

impl<T> Index<ChunkPosition> for SharedSector<T> where T: Packed {
	type Output = RwLock<Option<T>>;

	fn index(&self, index: ChunkPosition) -> &Self::Output {
		&self.chunks[index.yzx() as usize]
	}
}