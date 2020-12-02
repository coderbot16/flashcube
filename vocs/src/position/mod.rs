mod chunk;
mod layer;
mod column;
mod quad;
mod direction;

/// Global positioning to complement the local positions.
///
/// Unlike vanilla, positioning has well defined limits. rs25 defines a Minecraft World as a sparse 4294967296 by 256 by 4294967296 volume of blocks.
/// This can also be represented as a sparse 268435456 by 16 by 268435456 volume of chunks.
/// Finally, it can be represented as a sparse 16777216 by 16777216 grid of sectors.
mod global;

pub use self::chunk::ChunkPosition;
pub use self::layer::LayerPosition;
pub use self::column::ColumnPosition;
pub use self::quad::QuadPosition;
pub use self::direction::{Offset, Dir, Axis, StaticAxis, StaticDirection, dir};
pub use self::global::{GlobalPosition, GlobalChunkPosition, GlobalColumnPosition, GlobalSectorPosition};