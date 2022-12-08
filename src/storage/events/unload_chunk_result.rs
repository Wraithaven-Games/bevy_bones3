//! A builder-style handler for processing the action result for unloading
//! chunks within a chunk storage container.

use anyhow::Result;
use bevy::prelude::{Entity, EventWriter, IVec3};

use super::ChunkUnloadEvent;

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
