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

pub use self::mask::{Mask, ChunkMask, LayerMask};