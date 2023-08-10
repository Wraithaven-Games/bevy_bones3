//! This module defines the VoxelWorld component that contains an infinite, 3D
//! grid of block data and describes how pieces of that data should be handled
//! and processed by other systems.

use bevy::prelude::*;
use bevy::utils::HashMap;

/// A voxel world defines an infinite, 3D grid of blocks that are stored within
/// chunks as child components of this world.
#[derive(Debug, Component)]
pub struct VoxelWorld {
    /// The entity ID of the block list to reference for reading block type
    /// information. All block IDs stored within this world are assumed to point
    /// to a block within this list.
    ///
    /// Using block IDs from other block lists might lead to crashes or other
    /// unexpected behaviors.
    block_list: Entity,

    /// An internal pointer handler for quickly finding the entity of a chunk
    /// based off of its chunk coordinates.
    chunks: HashMap<IVec3, Entity>,
}

impl VoxelWorld {
    /// Creates a new voxel world component.
    pub fn new(block_list: Entity) -> Self {
        Self {
            block_list,
            chunks: HashMap::new(),
        }
    }

    /// Gets the block list that is used by this voxel world.
    pub fn block_list(&self) -> Entity {
        self.block_list
    }

    /// Gets the entity of a chunk based off of it's coordinates.
    ///
    /// If the chunk does not exist, `None` is returned.
    pub fn get_chunk_id(&self, coords: IVec3) -> Option<&Entity> {
        let coords = coords >> 4;
        self.chunks.get(&coords)
    }

    /// Inserts, (or replaces) the chunk_id for the chunk at the given chunk
    /// coordinates within the local chunk pointer cache.
    pub(super) fn update_chunk_id(&mut self, coords: IVec3, chunk_id: Entity) {
        let coords = coords >> 4;
        self.chunks.insert(coords, chunk_id);
    }
}
