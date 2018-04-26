// TODO: Basic ECS: Allow common tasks to fall under common 7 types, but provide extension with specs. This can avoid dynamic type casting in most cases.
// bool: ChunkMask
// u4: ChunkNibbles
// u8: [u8; 4096]
// uXX: Packed
// f32: [f32; 4096]
// f64: [f64; 4096]
// String: HashMap<ChunkPosition, String>
// Entity: A complex struct stored in the local specs ECS.

use position::{LayerPosition, ChunkPosition};

/// A component usable in a comoponent system.
pub trait Component: Sized + Clone + Default {
	/// Dense storage in a 16x16x16 chunk.
	type Chunk: ChunkStorage<Self> + Default;
	/// Dense storage in a 16x16 layer.
	type Layer: LayerStorage<Self> + Default;
	/// Dense storage of an unknown length.
	type Bulk;
	/// Sparse storage of an unknown length.
	type BulkSparse;
}

pub trait ChunkStorage<V> where V: Clone {
	/// Gets the value at the position.
	fn get(&self, position: ChunkPosition) -> V;

	/// Sets the value at the position.
	fn set(&mut self, position: ChunkPosition, value: V);

	/// Fills the storage with the value.
	fn fill(&mut self, value: V) {
		for index in 0..4096 {
			let position = ChunkPosition::from_yzx(index);

			self.set(position, value.clone())
		}
	}
}

pub trait LayerStorage<V> where V: Clone {
	/// Gets the value at the position.
	fn get(&self, position: LayerPosition) -> V;

	/// Sets the value at the position.
	fn set(&mut self, position: LayerPosition, value: V);

	/// Fills the storage with the value.
	fn fill(&mut self, value: V) {
		for index in 0..256u16 {
			let position = LayerPosition::from_zx(index as u8);

			self.set(position, value.clone())
		}
	}
}