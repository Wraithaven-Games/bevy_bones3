//! Handler components for storing data within a chunk.

use bevy::prelude::*;

use crate::math::Region;

/// A blanket trait for data types that can be safely stored within a voxel
/// world.
pub trait BlockData: Default + Copy + Send + Sync + 'static {}
impl<T> BlockData for T where T: Default + Copy + Send + Sync + 'static {}

/// A storage component for containing a 16x16x16 grid of block data. This is
/// usually intended to be used on a voxel chunk component.
///
/// By default it is filled with the default value for `T`.
#[derive(Debug, Component, Reflect)]
pub struct VoxelStorage<T>
where
    T: BlockData,
{
    /// The block data array for this chunk.
    #[reflect(ignore)]
    blocks: Option<Box<[T; 4096]>>,
}

impl<T> Default for VoxelStorage<T>
where
    T: BlockData,
{
    fn default() -> Self {
        Self {
            blocks: None,
        }
    }
}

impl<T> VoxelStorage<T>
where
    T: BlockData,
{
    /// Gets the block data at the local grid coordinates within this storage
    /// component.
    ///
    /// If the coordinates are outside of the 16x16x16 grid, they are wrapped
    /// back ground to the other side.
    pub fn get_block(&self, local_pos: IVec3) -> T {
        let index = Region::CHUNK.point_to_index(local_pos & 15).unwrap();
        match &self.blocks {
            Some(arr) => arr[index],
            None => T::default(),
        }
    }

    /// Sets the block data at the local grid coordinates within this storage
    /// component.
    ///
    /// If the coordinates are outside of the 16x16x16 grid, they are wrapped
    /// back ground to the other side.
    pub fn set_block(&mut self, local_pos: IVec3, data: T) {
        let index = Region::CHUNK.point_to_index(local_pos & 15).unwrap();
        match &mut self.blocks {
            Some(arr) => arr[index] = data,
            None => {
                let mut chunk = Box::new([T::default(); 4096]);
                chunk[index] = data;
                self.blocks = Some(chunk);
            },
        }
    }
}
