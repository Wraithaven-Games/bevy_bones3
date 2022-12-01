//! This module contains events that can be triggered by the `bevy_bones3`
//! crate.

use bevy::prelude::{Entity, IVec3};

/// An event that is triggered when a chunk is loaded within a world.
///
/// A chunk load event is defined as when a chunk is first initialized within a
/// voxel world. By default, chunks are all loaded with default data values.
/// This event may be triggered before the chunk is populated by other systems.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkLoadEvent {
    /// The entity that contains the voxel world where this event took place.
    pub world: Entity,

    /// The coordinates of the chunk that was loaded.
    pub chunk_coords: IVec3,
}

/// An event that is triggered when a chunk is unloaded from a voxel world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkUnloadEvent {
    /// The entity that contains the voxel world where this event took place.
    pub world: Entity,

    /// The coordinates of the chunk that was unloaded.
    pub chunk_coords: IVec3,
}
