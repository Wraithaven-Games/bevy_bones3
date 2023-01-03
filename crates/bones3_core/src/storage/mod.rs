//! Contains the implementation for VoxelWorld data storage and manipulation.
//!
//! This module only refers to the current in-memory state of the world.
//! Unloaded sections of the world must be loaded before they can be properly
//! manipulated.

mod chunk;
pub(crate) mod chunk_pointers;
mod data;

pub use chunk::*;
pub use data::*;
