//! A voxel chunk component.

use bevy::prelude::*;

/// A voxel world marker component.
#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct VoxelWorld;

/// A pointer to indicate the coordinates of a chunk.
#[derive(Debug, Component, Reflect, PartialEq, Eq, Hash)]
pub struct VoxelChunk {
    /// The world id this chunk is in.
    world_id: Entity,

    /// The coordinates of this chunk.
    chunk_coords: IVec3,
}

impl VoxelChunk {
    /// Creates a new voxel chunk at the given chunk coordinates.
    pub(crate) fn new(world_id: Entity, chunk_coords: IVec3) -> Self {
        Self {
            world_id,
            chunk_coords,
        }
    }

    ///  Gets the world id of this chunk.
    pub fn world_id(&self) -> Entity {
        self.world_id
    }

    /// Gets the coordinates of this chunk.
    pub fn chunk_coords(&self) -> IVec3 {
        self.chunk_coords
    }
}
