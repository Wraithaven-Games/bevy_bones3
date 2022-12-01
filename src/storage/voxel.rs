//! The trait definitions for how voxel storage devices are implemented.

use anyhow::Result;
use bevy::prelude::IVec3;

/// A blanket trait for data types that can be safely stored within a voxel
/// world.
pub trait BlockData: Default + Copy + Send + Sync + 'static {}
impl<T> BlockData for T where T: Default + Copy + Send + Sync + 'static {}

/// Defines that block data may be read from, or written this object based on
/// block coordinates.
pub trait VoxelStorage<T: BlockData> {
    /// Gets the block data at the given block coordinates within this data
    /// container.
    ///
    /// This function returns an error if the block coordinates lie outside of
    /// the bounds of this container.
    fn get_block(&self, block_coords: IVec3) -> Result<T>;

    /// Sets the block data at the given block coordinates within this data
    /// container.
    ///
    /// This function returns an error if the block coordinates lie outside of
    /// the bounds of this container.
    fn set_block(&mut self, block_coords: IVec3, data: T) -> Result<()>;
}

/// Defines that continuous chunks of data maybe be loaded and unloaded within
/// this data container.
pub trait ChunkLoad {
    /// Creates a new, empty chunk of block data at the given chunk coordinates
    /// within this data container.
    ///
    /// This function returns an error if the chunk coordinates lie outside of
    /// the bounds of this container, or if there is already a chunk loaded
    /// at the given chunk coordinates.
    fn init_chunk(&mut self, chunk_coords: IVec3) -> Result<()>;

    /// Unloads the chunk data at the given chunk coordinates.
    ///
    /// This function returns an error if the chunk coordinates lie outside of
    /// the bounds of this container, or if there is no chunks loaded at the
    /// given chunk coordinates.
    fn unload_chunk(&mut self, chunk_coords: IVec3) -> Result<()>;

    /// Checks if there is currently a chunk loaded at the given chunk
    /// coordinates or not.
    ///
    /// This function returns true if there is a chunk present at the given
    /// coordinates, and false if there is not. This function will return an
    /// error if the chunk coordinates lie outside of the bounds of this
    /// container.
    fn is_loaded(&self, chunk_coords: IVec3) -> Result<bool>;
}
