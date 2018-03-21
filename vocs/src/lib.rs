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
//! opportunity to experiment with a new architecture. But, we must first explode the
//! design of Vanilla Minecraft:
//!
//!  * Block classes defining the behavior of each voxel type
//!  * BlockEntity classes for storing additional complex block information
//!  * BlockState for fetching small properties from a block ID
//!  * ChunkSection for storing the staticly defined arrays for storage, as well as the BlockEntity list.
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

// Execution dispatcher
extern crate shred;

pub mod world;
pub mod storage;
pub mod position;