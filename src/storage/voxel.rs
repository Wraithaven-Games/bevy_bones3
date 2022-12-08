//! The trait definitions for how voxel storage devices are implemented.

use anyhow::Result;
use bevy::prelude::IVec3;

use super::events::InitChunkResult;
use super::{UnloadAllChunksResult, UnloadChunkResult, VoxelWorldSlice};
use crate::math::Region;

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
    /// If the give block coordinates are outside of the bounds of this storage
    /// container, or are within an unloaded chunk, then the default value
    /// for T is returned.
    fn get_block(&self, block_coords: IVec3) -> T;

    /// Sets the block data at the given block coordinates within this data
    /// container.
    ///
    /// This function returns an error if the block coordinates lie outside of
    /// the bounds of this container or are not in a fully loaded chunk.
    fn set_block(&mut self, block_coords: IVec3, data: T) -> Result<()>;
}

/// An extension for voxel storage container that allows for groups of blocks to
/// be read from and written to at a time for performance improvements.
pub trait VoxelStorageRegion<T: BlockData>: VoxelStorage<T> {
    /// Gets a slice of the voxel storage container, cloning over all voxel data
    /// within the region bounds.
    ///
    /// This method creates a new voxel world slice based on the given requested
    /// region selection, and returns it. This approach is functionally
    /// identical to reading each block one by one, via
    /// [`VoxelStorage::get_block`], but is much faster when reading a large
    /// number of blocks that are near one another.
    ///
    /// If the indicated region intersects areas outside of the container, those
    /// locations within the returned region are set to the default value of T.
    fn get_slice(&self, region: Region) -> VoxelWorldSlice<T>;

    /// Sets all blocks within a region of this voxel storage container based on
    /// the corresponding data from the provided voxel world slice.
    ///
    /// This method is functionally identical to setting each block within the
    /// region one by one, but is more performant for larger regions.
    ///
    /// This function will copy the data stored within the voxel world slice and
    /// place that data within this voxel container at the same coordinate
    /// location. If the provided world slice contains sections that are out
    /// of bounds of this voxel container, an error is returned.
    /// This functionality is similar to [`VoxelStorage::set_block`], where an
    /// error will also be returned by attempting to edit blocks out of the
    /// loaded world bounds.
    fn fill_slice(&mut self, slice: VoxelWorldSlice<T>) -> Result<()>;
}

/// Defines that continuous chunks of data maybe be loaded and unloaded within
/// this data container.
pub trait ChunkStorage<W, T>
where
    W: ChunkStorage<W, T>,
    T: BlockData, {
    /// Creates a new, empty chunk of block data at the given chunk coordinates
    /// within this data container.
    ///
    /// This function returns an error if the chunk coordinates lie outside of
    /// the bounds of this container, or if there is already a chunk loaded
    /// at the given chunk coordinates.
    fn init_chunk(&mut self, chunk_coords: IVec3) -> InitChunkResult<W, T>;

    /// Unloads the chunk data at the given chunk coordinates.
    ///
    /// This function returns an error if the chunk coordinates lie outside of
    /// the bounds of this container, or if there is no chunks loaded at the
    /// given chunk coordinates.
    fn unload_chunk(&mut self, chunk_coords: IVec3) -> UnloadChunkResult;

    /// Unloads all chunks within this world.
    ///
    /// This function does nothing if there are no loaded chunks within this
    /// chunk container.
    fn unload_all_chunks(&mut self) -> UnloadAllChunksResult;

    /// Checks if there is currently a chunk loaded at the given chunk
    /// coordinates or not.
    fn is_chunk_loaded(&self, chunk_coords: IVec3) -> bool;
}
