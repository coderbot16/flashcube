use position::{LayerPosition, ChunkPosition};

mod internal;
mod setter;

pub use self::internal::{PackedIndex, PackedStorage};

pub type ChunkPacked = self::internal::PackedStorage<ChunkPosition>;
pub type LayerPacked = self::internal::PackedStorage<LayerPosition>;

pub type PackedBlockStorage<P> = self::internal::PackedStorage<P>;