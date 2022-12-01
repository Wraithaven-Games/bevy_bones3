//! This module contains the Bevy component that implements the chunk anchor.

use bevy::prelude::*;

/// Defines an anchor within a world that forces a radius of chunks around
/// itself to stay loaded.
///
/// This component relies on the Position component in order to function.
#[derive(Debug, Clone, PartialEq, Eq, Reflect, FromReflect, Component, Default)]
#[reflect(Component, PartialEq)]
pub struct ChunkAnchor {
    /// The world that this chunk anchor is pinned to.
    pub world: Option<Entity>,

    /// The radius, in chunks, that are triggered to load around the chunk
    /// anchor.
    ///
    /// A value of 0 will only trigger a single chunk to remain loaded.
    pub radius: u16,

    /// The maximum number of chunks around this anchor that are allowed to
    /// remain loaded before being considered out of range.
    ///
    /// A value of 0 will only allow for a single chunk to be considered within
    /// range of this anchor.
    pub max_radius: u16,
}

impl ChunkAnchor {
    /// Creates a new chunk anchor instance.
    ///
    /// The world entity must be an entity that contains the VoxelChunkStates
    /// component.
    pub fn new(world: Entity, radius: u16, max_radius: u16) -> Self {
        Self {
            world: Some(world),
            radius,
            max_radius,
        }
    }
}
