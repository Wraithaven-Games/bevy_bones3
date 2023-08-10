//! This module contains implementations for system parameters and events that
//! handle reading and writing to voxel worlds.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{Block, BlockList, VoxelChunk, VoxelWorld};

/// A read-only system parameter for reading block data from a voxel world.
#[derive(SystemParam)]
pub struct VoxelData<'w, 's> {
    /// A query of all voxel chunks.
    chunks: Query<'w, 's, &'static VoxelChunk>,

    /// A query of all block lists.
    block_lists: Query<'w, 's, &'static BlockList>,
}

impl<'w, 's> VoxelData<'w, 's> {
    /// Gets a read-only reference to the block data stored within the block at
    /// the given coordinates.
    ///
    /// If the block at the given coordinates does not exist or has not been
    /// specified, then `None` is returned.
    pub fn block(&self, world: &VoxelWorld, coords: IVec3) -> Option<&Block> {
        let chunk_id = world.get_chunk_id(coords)?;
        let chunk = self.chunks.get(*chunk_id).ok()?;
        let block_id = chunk[coords];
        let block_list = self.block_lists.get(world.block_list()).ok()?;
        Some(&block_list[block_id])
    }
}
