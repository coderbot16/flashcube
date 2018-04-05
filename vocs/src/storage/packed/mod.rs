use position::{LayerPosition, ChunkPosition};

mod internal;

pub use self::internal::PackedIndex;

pub type ChunkPacked = self::internal::PackedStorage<ChunkPosition>;
pub type LayerPacked = self::internal::PackedStorage<LayerPosition>;

pub type PackedBlockStorage<P> = self::internal::PackedStorage<P>;