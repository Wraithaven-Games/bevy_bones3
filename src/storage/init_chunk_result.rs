//! A builder-style handler for processing the action result for initializing a
//! new chunk within a chunk storage container.

use std::marker::PhantomData;

use anyhow::{bail, Result};
use bevy::prelude::{Entity, EventWriter, IVec3};

use super::{BlockData, ChunkStorage, VoxelStorageRegion, VoxelWorldSlice};
use crate::events::ChunkLoadEvent;
use crate::math::Region;

/// The return type for [`ChunkStorage::init_chunk`], for the purpose of
/// chaining together actions.
#[must_use]
pub struct InitChunkResult<'a, W: ChunkStorage<W, T>, T: BlockData> {
    /// A reference to the chunk storage container that generated this chunk
    /// result.
    storage: &'a mut W,

    /// The coordinates of the chunk that was created.
    chunk_coords: IVec3,

    /// The result output of the action.
    result: Result<()>,

    /// The phantom data for T.
    _phantom: PhantomData<T>,
}

impl<'a, W: ChunkStorage<W, T>, T: BlockData> InitChunkResult<'a, W, T> {
    /// Creates a new InitChunkResult from the given function output and chunk
    /// coordinates.
    pub fn new(storage: &'a mut W, result: Result<()>, chunk_coords: IVec3) -> Self {
        Self {
            storage,
            chunk_coords,
            result,
            _phantom: PhantomData::default(),
        }
    }

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
        match self.result {
            Ok(_) => {
                event_writer.send(ChunkLoadEvent {
                    world,
                    chunk_coords: self.chunk_coords,
                });
                Ok(())
            },
            Err(err) => Err(err),
        }
    }

    /// Gets the result status of this chunk initialization action.
    pub fn get_result(&self) -> &Result<()> {
        &self.result
    }

    /// Converts this ChunkInitResult into a Result<()> type for error checking.
    pub fn into_result(self) -> Result<()> {
        self.result
    }

    /// Gets the chunk coords used.
    pub fn get_chunk_coords(&self) -> IVec3 {
        self.chunk_coords
    }
}

impl<'a, W: ChunkStorage<W, T> + VoxelStorageRegion<T>, T: BlockData> InitChunkResult<'a, W, T> {
    /// Fills the newly generated chunk with the given block data.
    ///
    /// The provided world slice much match the region bounds of the initialized
    /// chunk, or an error is returned.
    ///
    /// If the chunk failed to initialize and currently has an error status,
    /// then that error is returned and no further action is taken.
    pub fn fill_blocks(self, slice: VoxelWorldSlice<T>) -> Result<Self> {
        self.result?;

        let expected_region = Region::CHUNK.shift(self.chunk_coords << 4);
        if slice.get_region() != expected_region {
            bail!(
                "The provided world slice at {} does not match the expected region at {}",
                slice.get_region(),
                expected_region
            );
        }

        self.storage.fill_slice(slice)?;
        Ok(Self::new(self.storage, Ok(()), self.chunk_coords))
    }
}
