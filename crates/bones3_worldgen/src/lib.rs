//! Contains systems and components that are useful for generating parts of an
//! infinite voxel world on demand as chunks load.
//!
//! This module also handles the automatic loading and unloading of chunks
//! within a voxel world based off a given chunk anchor's position and effect
//! radius.
//!
//! This module requires the `world_gen` feature to use.

#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

use bevy::prelude::*;
use bones3_core::storage::BlockData;
use bones3_core::util::anchor::{ChunkAnchorPlugin, ChunkAnchorSet};

use crate::ecs::{components, systems};

pub mod ecs;

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
        app.register_type::<components::WorldGeneratorHandler<T>>()
            .register_type::<components::LoadChunkTask<T>>()
            .register_type::<components::PendingLoadChunkTask>()
            .add_plugins(ChunkAnchorPlugin::<WorldGenAnchor>::default())
            .add_systems(
                Update,
                (
                    systems::queue_chunks::<T>.in_set(WorldGenSet::QueueChunks),
                    systems::push_chunk_async_queue::<T>.in_set(WorldGenSet::StartAsyncTask),
                    systems::finish_chunk_loading::<T>.in_set(WorldGenSet::FinishAsyncTask),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    systems::create_chunk_entities.in_set(WorldGenSet::CreateChunks),
                    systems::unload_chunks.in_set(WorldGenSet::UnloadChunks),
                ),
            )
            .configure_set(
                PostUpdate,
                WorldGenSet::CreateChunks.after(ChunkAnchorSet::UpdateCoords),
            )
            .configure_set(
                PostUpdate,
                WorldGenSet::UnloadChunks.after(ChunkAnchorSet::UpdatePriorities),
            );
    }
}

#[derive(Debug, SystemSet, PartialEq, Eq, Hash, Clone, Copy)]
pub enum WorldGenSet {
    CreateChunks,
    UnloadChunks,
    QueueChunks,
    StartAsyncTask,
    FinishAsyncTask,
}

#[derive(Default, Reflect)]
pub struct WorldGenAnchor;
