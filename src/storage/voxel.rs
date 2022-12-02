//! The trait definitions for how voxel storage devices are implemented.

use anyhow::Result;
use bevy::prelude::{Entity, EventWriter, IVec3};

use crate::prelude::{ChunkLoadEvent, ChunkUnloadEvent};

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
pub trait ChunkStorage {
    /// Creates a new, empty chunk of block data at the given chunk coordinates
    /// within this data container.
    ///
    /// This function returns an error if the chunk coordinates lie outside of
    /// the bounds of this container, or if there is already a chunk loaded
    /// at the given chunk coordinates.
    fn init_chunk(&mut self, chunk_coords: IVec3) -> InitChunkResult;

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
    ///
    /// This function returns true if there is a chunk present at the given
    /// coordinates, and false if there is not. This function will return an
    /// error if the chunk coordinates lie outside of the bounds of this
    /// container.
    fn is_chunk_loaded(&self, chunk_coords: IVec3) -> Result<bool>;
}

/// The return type for [`ChunkStorage::init_chunk`], for the purpose of
/// chaining together actions.
#[must_use]
pub struct InitChunkResult(pub Result<IVec3>);

impl InitChunkResult {
    /// Triggers the chunk load event based on the results of the chunk_init
    /// output.
    ///
    /// If the chunk_init function failed to load a new chunk, then no event is
    /// triggered.
    pub fn call_event(
        self,
        event_writer: &mut EventWriter<ChunkLoadEvent>,
        world: Entity,
    ) -> Result<()> {
        match self.0 {
            Ok(chunk_coords) => {
                event_writer.send(ChunkLoadEvent {
                    world,
                    chunk_coords,
                });
                Ok(())
            },
            Err(err) => Err(err),
        }
    }

    /// Converts this ChunkInitResult into a Result<()> type for error checking.
    pub fn into_result(self) -> Result<()> {
        match self.0 {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// The return type for [`ChunkStorage::unload_chunk`], for the purpose of
/// chaining together actions.
#[must_use]
pub struct UnloadChunkResult(pub Result<IVec3>);

impl UnloadChunkResult {
    /// Triggers the chunk load event based on the results of the chunk_init
    /// output.
    ///
    /// If the chunk_init function failed to load a new chunk, then no event is
    /// triggered.
    pub fn call_event(
        self,
        event_writer: &mut EventWriter<ChunkUnloadEvent>,
        world: Entity,
    ) -> Result<()> {
        match self.0 {
            Ok(chunk_coords) => {
                event_writer.send(ChunkUnloadEvent {
                    world,
                    chunk_coords,
                });
                Ok(())
            },
            Err(err) => Err(err),
        }
    }

    /// Converts this ChunkInitResult into a Result<()> type for error checking.
    pub fn into_result(self) -> Result<()> {
        match self.0 {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// The return type for [`ChunkStorage::unload_all_chunks`], for the purpose of
/// chaining together actions.
pub struct UnloadAllChunksResult(pub Vec<IVec3>);

impl UnloadAllChunksResult {
    /// Triggers the chunk load event based on the results of the chunk_init
    /// output.
    ///
    /// If the chunk_init function failed to load a new chunk, then no event is
    /// triggered.
    pub fn call_event(self, event_writer: &mut EventWriter<ChunkUnloadEvent>, world: Entity) {
        event_writer.send_batch(self.0.into_iter().map(|chunk_coords| {
            ChunkUnloadEvent {
                world,
                chunk_coords,
            }
        }));
    }
}
