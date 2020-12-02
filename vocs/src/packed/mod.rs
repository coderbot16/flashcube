use crate::position::{LayerPosition, CubePosition};

mod internal;
mod setter;

pub use self::internal::{PackedIndex, PackedStorage};
pub use self::setter::Setter;

pub type PackedCube = self::internal::PackedStorage<CubePosition>;
pub type LayerPacked = self::internal::PackedStorage<LayerPosition>;

pub type PackedBlockStorage<P> = self::internal::PackedStorage<P>;