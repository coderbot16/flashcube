#![forbid(unsafe_code)]

//! # `vocs`: Voxel Component System
//!
//! An application of ECS to a voxel world.
//! There are many properties that make standard ECS implementations like
//! `specs` unsuitable for a dense voxel world, but their concepts still apply
//! in many forms.
//!
//! # Rationale
//!
//! This library is mainly intended to support an implementation of a Minecraft-compatible
//! voxel engine. With a ground-up reimplementation of Minecraft, there is an
//! opportunity to experiment with a new architecture. But, we must first explore the
//! design of Vanilla Minecraft:
//!
//!  * Block classes defining the behavior of each voxel type
//!  * BlockEntity classes for storing additional complex block information
//!  * BlockState for fetching small properties from a block ID
//!  * ChunkSection for storing the statically defined arrays for storage, as well as the BlockEntity list.
//!
//! All of these features are natural extensions of a simple base, but are
//! not optimal in many cases:
//!
//!  * BlockEntities are the cause of significant lag and memory use for many reasons
//!  * BlockState involves much allocation of temporaries, increasing memory pressure
//!  * Block only supports limited behaviors and properties.
//!
//! So, `vocs` proposes an alternative architecture.
//!
//! # Architecture
//!
//! All of the data associated with a Chunk are components:
//!  * BlockLight / SkyLight
//!  * Block IDs
//!  * BlockEntity properties
//!
//! All of the code associated with simulating Blocks and BlockEntities
//! are moved into Systems.
//!  * Lighting
//!  * General block behavior
//!  * Block entity behavior (ex: Furnace ticking, Piston movement)
//!
//! Where `vocs` differs from a standard ECS is in how it handles certain aspects. For example,
//! there is no "entity ID" system, as the block position is the unique identifier.
//!

// Variable length bit collections
extern crate bit_vec;

// Efficient and fine-grained spin locks
extern crate spin;

// Fast hash map
extern crate rustc_hash;

// Access multiple distinct hash map entries at same time
extern crate splitmut;

pub mod world;
pub mod position;
// TODO[not yet implemented]: pub mod system;

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

pub mod indexed;

/// Chunk & Layer sized arrays without packing.
pub mod unpacked;

/// Ways of viewing a collection of chunks.
pub mod view;

/// The core of the voxel component system.
pub mod component;

pub mod sparse;
