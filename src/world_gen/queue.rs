//! The chunk loading queue to determine which chunks need to be loaded and how
//! soon they need to be loaded.

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use ordered_float::OrderedFloat;

use super::{ChunkAnchor, WorldGeneratorHandler};
use crate::prelude::{VoxelCommands, VoxelQuery};
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

/// This function will trigger all of the chunks that are requested by chunk
/// anchors to begin loading.
pub fn load_chunks_async<T>(
    chunks: VoxelQuery<()>,
    mut anchors: Query<(Entity, &Transform, &mut ChunkAnchor)>,
    mut commands: Commands,
) where
    T: BlockData,
{
    for (anchor_id, transform, mut anchor) in anchors.iter_mut() {
        let Some(world_id) = anchor.get_world() else {
            continue;
        };

        if !chunks.has_world(world_id) {
            warn!(
                ?world_id,
                ?anchor_id,
                "Chunk anchor points to world that doesn't exist",
            );
            anchor.set_world(None);
            continue;
        };

        #[cfg(feature = "trace")]
        let profiler_guard = info_span!("world_gen", target = "find_new_chunks").entered();

        let anchor_pos = transform.translation.as_ivec3() >> 4;
        for chunk_coords in anchor.iter(anchor_pos) {
            if chunks.get_chunk(world_id, chunk_coords).is_err() {
                commands
                    .voxel_world(world_id)
                    .unwrap()
                    .spawn_chunk(chunk_coords, PendingLoadChunkTask);
            };
        }

        #[cfg(feature = "trace")]
        profiler_guard.exit();
    }
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
    let profiler_guard = info_span!("world_gen", target = "find_next_pending_chunk").entered();

    let next_chunk = pending_tasks.iter().max_by_key(|q| {
        let mut priority = f32::NEG_INFINITY;

        for (transform, anchor) in anchors.iter() {
            let pos = transform.translation.as_ivec3() >> 4;
            let anchor_priority = anchor.get_priority(pos, q.1.chunk_coords());
            priority = f32::max(priority, anchor_priority);
        }

        OrderedFloat(priority)
    });

    #[cfg(feature = "trace")]
    profiler_guard.exit();

    let Some((chunk_id, pending_task)) = next_chunk else {
        return;
    };

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

/// This system takes in all active async chunk loading tasks and, for each one
/// that is finished, push the results to the target voxel chunk.
pub fn finish_chunk_loading<T: BlockData>(
    mut chunks: VoxelQuery<Entity, With<VoxelStorage<T>>>,
    mut load_chunk_tasks: Query<(Entity, &mut LoadChunkTask<T>, &VoxelChunk)>,
    mut commands: Commands,
) {
    #[cfg(feature = "trace")]
    let profiler_guard = info_span!("world_gen", target = "finalize_chunks").entered();

    for (chunk_id, mut task, chunk_meta) in load_chunk_tasks.iter_mut() {
        let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        commands
            .entity(chunk_id)
            .remove::<LoadChunkTask<T>>()
            .insert(chunk_data);

        #[cfg(feature = "meshing")]
        chunks
            .remesh_chunk_neighbors(chunk_meta.world_id(), chunk_meta.chunk_coords())
            .unwrap();
    }

    #[cfg(feature = "trace")]
    profiler_guard.exit();
}
