//! The chunk loading queue to determine which chunks need to be loaded and how
//! soon they need to be loaded.

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use sort_by_derive::SortBy;

use super::{ChunkAnchor, WorldGeneratorHandler};
use crate::prelude::{VoxelCommands, VoxelQueryError};
use crate::storage::{BlockData, VoxelChunk, VoxelStorage, VoxelWorld};

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
        #[cfg(feature = "trace")]
        let _profiler_guard = info_span!("world_gen", target = "force_load_chunks").entered();

        let world_id = anchor.get_world();
        if !commands.has_world(world_id) {
            continue;
        };

        let anchor_pos = transform.translation.as_ivec3() >> 4;
        let Some(chunk_iter) = anchor.iter_force(anchor_pos) else {
            continue;
        };

        for chunk_coords in chunk_iter {
            match commands.find_chunk(world_id, chunk_coords, true) {
                Ok(_) => continue,
                Err(VoxelQueryError::ChunkNotFound(..)) => {
                    let block_data = match generators.get(world_id).ok().map(|g| g.generator()) {
                        Some(gen) => gen.generate_chunk(chunk_coords),
                        None => VoxelStorage::<T>::default(),
                    };

                    commands
                        .spawn_chunk(world_id, chunk_coords, block_data)
                        .unwrap();

                    #[cfg(feature = "meshing")]
                    commands
                        .remesh_chunk_neighbors(world_id, chunk_coords)
                        .unwrap();

                    #[cfg(feature = "physics")]
                    commands.rebuild_collision(world_id, chunk_coords).unwrap();
                },
                Err(err) => panic!("Unexpected state: {:?}", err),
            }
        }
    }
}

/// This function will trigger all of the chunks that are requested by chunk
/// anchors to begin loading.
pub fn load_chunks_async(anchors: Query<(&Transform, &ChunkAnchor)>, mut commands: VoxelCommands) {
    for (transform, anchor) in anchors.iter() {
        #[cfg(feature = "trace")]
        let _profiler_guard = info_span!("world_gen", target = "find_new_chunks").entered();

        let world_id = anchor.get_world();
        if !commands.has_world(world_id) {
            continue;
        };

        let anchor_pos = transform.translation.as_ivec3() >> 4;
        for chunk_coords in anchor.iter(anchor_pos) {
            match commands.find_chunk(world_id, chunk_coords, true) {
                Ok(_) => continue,
                Err(VoxelQueryError::ChunkNotFound(..)) => {
                    commands
                        .spawn_chunk(world_id, chunk_coords, PendingLoadChunkTask)
                        .unwrap();
                },
                Err(err) => panic!("Unexpected state: {:?}", err),
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

    #[cfg(feature = "trace")]
    let profiler_guard = info_span!("world_gen", target = "find_next_pending_chunks").entered();

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

    #[cfg(feature = "trace")]
    profiler_guard.exit();

    #[cfg(feature = "trace")]
    let _profiler_guard = info_span!("world_gen", target = "start_async_tasks").entered();

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
    mut load_chunk_tasks: Query<(&mut LoadChunkTask<T>, &VoxelChunk)>,
    mut commands: VoxelCommands,
) {
    #[cfg(feature = "trace")]
    let _profiler_guard = info_span!("world_gen", target = "finalize_chunks").entered();

    for (mut task, chunk_meta) in load_chunk_tasks.iter_mut() {
        #[cfg(feature = "trace")]
        let poll_task_guard = info_span!("world_gen", target = "poll_load_task").entered();

        let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        #[cfg(feature = "trace")]
        poll_task_guard.exit();

        #[cfg(feature = "trace")]
        let _single_prof_guard =
            info_span!("world_gen", target = "finalize_chunk", world_id = ?chunk_meta.world_id())
                .entered();

        commands
            .find_chunk(chunk_meta.world_id(), chunk_meta.chunk_coords(), false)
            .unwrap()
            .remove::<LoadChunkTask<T>>()
            .insert(chunk_data);

        #[cfg(feature = "meshing")]
        commands
            .remesh_chunk_neighbors(chunk_meta.world_id(), chunk_meta.chunk_coords())
            .unwrap();

        #[cfg(feature = "physics")]
        commands
            .rebuild_collision(chunk_meta.world_id(), chunk_meta.chunk_coords())
            .unwrap();
    }
}
