//! Contains systems and components that are useful for generating parts of an
//! infinite voxel world on demand as chunks load.
//!
//! This module also handles the automatic loading and unloading of chunks
//! within a voxel world based off a given chunk anchor's position and effect
//! radius.
//!
//! This module requires the `world_gen` feature to use.

use std::marker::PhantomData;

use bevy::prelude::*;
use bones3_core::storage::BlockData;
use prelude::{
    finish_chunk_loading,
    force_load_chunks,
    load_chunks_async,
    push_chunk_async_queue,
    ChunkAnchor,
    LoadChunkTask,
    PendingLoadChunkTask,
    WorldGeneratorHandler,
};
use prep_chunks::setup_chunk_transforms;

pub mod anchor;
pub mod generator;
pub mod prep_chunks;
pub mod queue;

/// Used to import common components and systems for Bones Cubed.
pub mod prelude {
    pub use super::anchor::*;
    pub use super::generator::*;
    pub use super::prep_chunks::*;
    pub use super::queue::*;
}

#[derive(Default)]
pub struct Bones3WorldGenPlugin<T>
where
    T: BlockData,
{
    /// Phantom data for T.
    _phantom: PhantomData<T>,
}

impl<T> Plugin for Bones3WorldGenPlugin<T>
where
    T: BlockData,
{
    fn build(&self, app: &mut App) {
        app.register_type::<ChunkAnchor>()
            .register_type::<WorldGeneratorHandler<T>>()
            .register_type::<LoadChunkTask<T>>()
            .register_type::<PendingLoadChunkTask>()
            .add_system(setup_chunk_transforms)
            .add_system(force_load_chunks::<T>)
            .add_system(load_chunks_async)
            .add_system(push_chunk_async_queue::<T>)
            .add_system(finish_chunk_loading::<T>);
    }
}
