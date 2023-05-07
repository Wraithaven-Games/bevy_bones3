//! The chunk loading queue to determine which chunks need to be loaded and how
//! soon they need to be loaded.

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bones3_core::prelude::{
    BlockData,
    VoxelChunk,
    VoxelCommands,
    VoxelQueryError,
    VoxelStorage,
    VoxelWorld,
};
#[cfg(feature = "meshing")]
use bones3_remesh::query::VoxelRemeshCommands;
use futures_lite::future;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use sort_by_derive::SortBy;

use crate::anchor::ChunkAnchor;
use crate::generator::WorldGeneratorHandler;

/// This component indicates that the chunk is currently being loaded in an
/// async task, and will have a voxel storage component replace this component
/// once it is done.
#[derive(Debug, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct LoadChunkTask<T: BlockData>(#[reflect(ignore)] Task<VoxelStorage<T>>);

/// A marker component that indicates that the target chunk is still waiting to
/// be loaded.
#[derive(Debug, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct PendingLoadChunkTask;

/// This system triggers all chunk anchors to force load nearby chunks as needed
/// to satisfy their force_radius values.
pub fn force_load_chunks<T>(
    anchors: Query<(&Transform, &ChunkAnchor)>,
    generators: Query<&WorldGeneratorHandler<T>, With<VoxelWorld>>,
    mut commands: VoxelCommands,
) where
    T: BlockData,
{
    for (transform, anchor) in anchors.iter() {
        let world_id = anchor.get_world();

        let Ok(mut world_commands) = commands.get_world(world_id) else {
            continue;
        };

        let anchor_pos = transform.translation.as_ivec3() >> 4;
        let Some(chunk_iter) = anchor.iter_force(anchor_pos) else {
            continue;
        };

        for chunk_coords in chunk_iter {
            match world_commands.get_chunk(chunk_coords) {
                Ok(_) => continue,
                Err(VoxelQueryError::ChunkNotFound(..)) => {
                    let block_data = match generators.get(world_id).ok().map(|g| g.generator()) {
                        Some(gen) => gen.generate_chunk(chunk_coords),
                        None => VoxelStorage::<T>::default(),
                    };

                    let chunk_commands = world_commands
                        .spawn_chunk(chunk_coords, block_data)
                        .unwrap();

                    #[cfg(feature = "meshing")]
                    chunk_commands.remesh_chunk_neighbors();
                },
                Err(err) => panic!("Unexpected state: {err:?}"),
            }
        }
    }
}

/// This function will trigger all of the chunks that are requested by chunk
/// anchors to begin loading.
pub fn load_chunks_async(anchors: Query<(&Transform, &ChunkAnchor)>, mut commands: VoxelCommands) {
    for (transform, anchor) in anchors.iter() {
        let world_id = anchor.get_world();
        let Ok(mut world_commands) = commands.get_world(world_id) else {
            continue;
        };

        let anchor_pos = transform.translation.as_ivec3() >> 4;
        for chunk_coords in anchor.iter(anchor_pos) {
            match world_commands.get_chunk(chunk_coords) {
                Ok(_) => continue,
                Err(VoxelQueryError::ChunkNotFound(..)) => {
                    world_commands
                        .spawn_chunk(chunk_coords, PendingLoadChunkTask)
                        .unwrap();
                },
                Err(err) => panic!("Unexpected state: {err:?}"),
            }
        }

        break;
    }
}

/// A temporary storage container for ordering pending chunks based on their
/// priority level.
#[derive(SortBy)]
struct PendingChunkPriority<T> {
    /// The current voxel query value  that contains the chunk information.
    query: T,

    /// The priority value of the chunk.
    #[sort_by]
    priority: OrderedFloat<f32>,
}

/// Moves queued chunk loading tasks to an active async chunk loading task.
pub fn push_chunk_async_queue<T>(
    active_tasks: Query<(Entity, &LoadChunkTask<T>)>,
    pending_tasks: Query<(Entity, &VoxelChunk), With<PendingLoadChunkTask>>,
    anchors: Query<(&Transform, &ChunkAnchor)>,
    generators: Query<&WorldGeneratorHandler<T>, With<VoxelWorld>>,
    mut commands: Commands,
) where
    T: BlockData,
{
    // TODO Move this value to a resource.
    /// The maximum number of async world generations tasks that can exist at
    /// once.
    const MAX_TASKS: i32 = 2;

    let available_slots = MAX_TASKS - active_tasks.iter().len() as i32;
    if available_slots <= 0 {
        return;
    }

    let next_chunks = pending_tasks
        .iter()
        .map(|q| {
            let mut priority = f32::NEG_INFINITY;

            for (transform, anchor) in anchors.iter() {
                let pos = transform.translation.as_ivec3() >> 4;
                let anchor_priority = anchor.get_priority(pos, q.1.chunk_coords());
                priority = f32::max(priority, anchor_priority);
            }

            PendingChunkPriority {
                query:    q,
                priority: OrderedFloat(priority),
            }
        })
        .k_smallest(available_slots as usize)
        .map(|p| p.query);

    for (chunk_id, pending_task) in next_chunks {
        let world_id = pending_task.world_id();
        let chunk_coords = pending_task.chunk_coords();
        match generators.get(world_id).ok().map(|g| g.generator()) {
            Some(gen) => {
                let pool = AsyncComputeTaskPool::get();
                let task = pool.spawn(async move { gen.generate_chunk(chunk_coords) });
                commands
                    .entity(chunk_id)
                    .remove::<PendingLoadChunkTask>()
                    .insert(LoadChunkTask(task));
            },

            None => {
                commands
                    .entity(chunk_id)
                    .remove::<PendingLoadChunkTask>()
                    .insert(VoxelStorage::<T>::default());
            },
        };
    }
}

/// This system takes in all active async chunk loading tasks and, for each one
/// that is finished, push the results to the target voxel chunk.
pub fn finish_chunk_loading<T: BlockData>(
    mut load_chunk_tasks: Query<(Entity, &mut LoadChunkTask<T>, &VoxelChunk)>,
    mut commands: VoxelCommands,
) {
    for (chunk_id, mut task, chunk_meta) in load_chunk_tasks.iter_mut() {
        let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        commands
            .commands()
            .entity(chunk_id)
            .remove::<LoadChunkTask<T>>()
            .insert(chunk_data);

        #[cfg(feature = "meshing")]
        commands
            .get_world(chunk_meta.world_id())
            .unwrap()
            .get_chunk(chunk_meta.chunk_coords())
            .unwrap()
            .remesh_chunk_neighbors();
    }
}
