// TODO: Basic ECS: Allow common tasks to fall under common 7 types, but provide extension with specs. This can avoid dynamic type casting in most cases.
// bool: BitCube
// u4: NibbleCube
// u8: [u8; 4096]
// uXX: Packed
// f32: [f32; 4096]
// f64: [f64; 4096]
// String: HashMap<CubePosition, String>
// Entity: A complex struct stored in the local specs ECS.

use crate::position::{LayerPosition, CubePosition};

/// A component usable in a comoponent system.
pub trait Component: Sized + Clone + Default {
	/// Dense storage in a 16x16x16 chunk.
	type Chunk: CubeStorage<Self> + Default;
	/// Dense storage in a 16x16 layer.
	type Layer: LayerStorage<Self> + Default;
	/// Dense storage of an unknown length.
	type Bulk;
}

pub trait CubeStorage<V> where V: Clone {
	/// Gets the value at the position.
	fn get(&self, position: CubePosition) -> V;

	/// Sets the value at the position.
	fn set(&mut self, position: CubePosition, value: V);

	/// Fills the storage with the value.
	fn fill(&mut self, value: V) {
		for position in CubePosition::enumerate() {
			self.set(position, value.clone())
		}
	}
}

pub trait LayerStorage<V> where V: Clone {
	/// Gets the value at the position.
	fn get(&self, position: LayerPosition) -> V;

	fn is_filled(&self, value: V) -> bool;

	/// Sets the value at the position.
	fn set(&mut self, position: LayerPosition, value: V);

	/// Fills the storage with the value.
	fn fill(&mut self, value: V) {
		for position in LayerPosition::enumerate() {
			self.set(position, value.clone())
		}
	}
}