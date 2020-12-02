use crate::packed::{PackedStorage, PackedIndex};

pub struct Setter<'s, P> where P: 's + PackedIndex {
	storage: &'s mut PackedStorage<P>,
	value: u32
}

impl<'s, P> Setter<'s, P> where P: 's + PackedIndex {
	pub fn new(storage: &'s mut PackedStorage<P>, value: u32) -> Self {
		Setter { storage, value }
	}

	pub fn set(&mut self, position: P) {
		self.storage.set(position, self.value)
	}

	pub fn get(&self, position: P) -> u32 {
		self.storage.get(position)
	}

	pub fn storage(&self) -> &PackedStorage<P> {
		&self.storage
	}
}

// TODO: MaskMut trait