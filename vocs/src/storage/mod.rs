/// Bit mask facilities. Very useful for marking things, such as:
///
///  * Free bitmaps in allocation
///  * Queues for block updates
///  * Marking exploded blocks in a chunk
///
/// There are 3 variants: LayerMask, ChunkMask, and an implementation of Mask for BitVec. LayerMask is for a 16x16 flat layer.
/// ChunkMask is for a 16x16x16 cube. Finally, BitVec is for arbitrary arrays of bits.
pub mod mask;

/// Bulk nibble storage. Useful for lighting data and one dimensional chunk coordinates, but not much else.
pub mod nibbles;

/// Variable length bit packed storage. Usually accessed using a palette.
pub mod packed;

pub use self::nibbles::ChunkNibbles;
pub use self::mask::{Mask, ChunkMask, LayerMask};

/// A component usable in an ECS.
pub trait Component {
	/// Dense storage in a 16x16x16 chunk.
	type ChunkStorage;
	/// Dense storage in a 16x16 layer.
	type LayerStorage;
	/// Dense storage of an unknown length.
	type DenseBulk;
	/// Sparse storage of an unknown length.
	type SparseBulk;
}

// TODO: Basic ECS: Allow common tasks to fall under common 7 types, but provide extension with specs. This can avoid dynamic type casting in most cases.
// bool: ChunkMask
// u4: ChunkNibbles
// u8: [u8; 4096]
// uXX: Packed
// f32: [f32; 4096]
// f64: [f64; 4096]
// String: HashMap<ChunkPosition, String>
// Entity: A complex struct stored in the local specs ECS.