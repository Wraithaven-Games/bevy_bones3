//! Represents a single 16x16x16 grid of voxels.


use anyhow::{bail, Result};
use bevy::prelude::*;

use super::voxel::VoxelStorage;
use super::BlockData;
use crate::math::region::Region;


/// A single 16x16x16 grid of data values that are stored within a voxel chunk.
/// The block data is stored in a fixed array on the heap.
#[derive(Debug)]
pub struct VoxelChunk<T: BlockData> {
    /// The block data array for this chunk.
    blocks: Box<[T; 4096]>,
}

impl<T: BlockData> Default for VoxelChunk<T> {
    fn default() -> Self {
        Self {
            blocks: Box::new([default(); 4096]),
        }
    }
}

impl<T: BlockData> VoxelStorage<T> for VoxelChunk<T> {
    fn get_block(&self, block_coords: IVec3) -> Result<T> {
        if let Ok(index) = Region::CHUNK.point_to_index(block_coords) {
            Ok(self.blocks[index])
        } else {
            bail!(
                "Block coordinates are outside of chunk bounds: {}",
                block_coords
            );
        }
    }

    fn set_block(&mut self, block_coords: IVec3, data: T) -> Result<()> {
        if let Ok(index) = Region::CHUNK.point_to_index(block_coords) {
            self.blocks[index] = data;
            Ok(())
        } else {
            bail!(
                "Block coordinates are outside of chunk bounds: {}",
                block_coords
            );
        }
    }
}
