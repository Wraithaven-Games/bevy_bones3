//! A self-contained region of block data. This acts as a single chunk of blocks
//! and is not nested.

use anyhow::Result;
use bevy::prelude::IVec3;

use super::{BlockData, VoxelStorage};
use crate::math::region::Region;

/// A self-contained region of blocks.
///
/// It acts as a small, finite storage container for voxel data and is much
/// faster for reading and writing voxel data than a voxel world, but cannot
/// change in size.
pub struct BlockRegion<T: BlockData> {
    /// The block data stored within this block region.
    blocks: Vec<T>,

    /// The region bounds for this block region.
    region: Region,
}

impl<T: BlockData> BlockRegion<T> {
    /// Creates a new block region based on the provided region bounds and fills
    /// it with the default value for T.
    pub fn new(region: Region) -> Self {
        Self {
            blocks: vec![T::default(); region.count()],
            region,
        }
    }

    /// Gets the region bounds.
    pub fn get_region(&self) -> Region {
        self.region
    }
}

impl<T: BlockData> VoxelStorage<T> for BlockRegion<T> {
    fn get_block(&self, block_coords: IVec3) -> Result<T> {
        let index = self.region.point_to_index(block_coords)?;
        Ok(self.blocks[index])
    }

    fn set_block(&mut self, block_coords: IVec3, data: T) -> Result<()> {
        let index = self.region.point_to_index(block_coords)?;
        self.blocks[index] = data;
        Ok(())
    }
}
