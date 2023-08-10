//! This module defines VoxelChunks, which are 16x16x16 collections of block IDs
//! that can be used to represent a small slice of an infinite voxel world.
//! Normally, these are handled completely internally.

use std::ops::Index;

use bevy::prelude::*;

use super::{BlockId, BlockList};

/// A chunk is a 16x16x16 grid of block IDs that make up a unique section of
/// a voxel world. These can be loaded and unloaded as needed.
#[derive(Debug, Component)]
pub struct VoxelChunk {
    /// The block coordinates of this chunk within the world.
    coords: IVec3,

    /// The 16x16x16 grid of block IDs.
    blocks: [BlockId; 16 * 16 * 16],
}

impl VoxelChunk {
    /// Creates a new voxel chunk instance using the given block coordinates.
    /// The chunk is filled using the default empty block value.
    pub(super) fn new(coords: IVec3) -> Self {
        let coords = (coords >> 4) << 4;

        Self {
            coords,
            blocks: [BlockList::default_block_id(); 16 * 16 * 16],
        }
    }

    /// Gets the block coordinates of this voxel chunk.
    pub fn coords(&self) -> IVec3 {
        self.coords
    }

    /// Replaces the block id at the given coordinates with a new value.
    ///
    /// If the block coordinates are outside of the bounds of this chunk, they
    /// are wrapped around to the other side.
    pub(super) fn set(&mut self, coords: IVec3, block_id: BlockId) {
        let coords = coords & 15;
        let index = coords.x * 16 * 16 + coords.y * 16 + coords.z;
        self.blocks[index as usize] = block_id;
    }
}

impl Index<IVec3> for VoxelChunk {
    type Output = BlockId;

    fn index(&self, coords: IVec3) -> &Self::Output {
        let coords = coords & 15;
        let index = coords.x * 16 * 16 + coords.y * 16 + coords.z;
        &self.blocks[index as usize]
    }
}
