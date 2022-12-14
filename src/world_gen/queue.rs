//! The chunk loading queue to determine which chunks need to be loaded and how
//! soon they need to be loaded.

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use itertools::Itertools;
use ordered_float::OrderedFloat;

use super::{ChunkAnchor, WorldGeneratorHandler};
use crate::prelude::{Region, RemeshChunk, VoxelCommands, VoxelQuery};
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

        let anchor_pos = transform.translation.as_ivec3() >> 4;
        for chunk_coords in anchor.iter(anchor_pos) {
            match chunks.get_chunk(world_id, chunk_coords) {
                Ok(()) => continue,
                Err(_) => {
                    commands
                        .voxel_world(world_id)
                        .unwrap()
                        .spawn_chunk(chunk_coords, PendingLoadChunkTask);
                },
            };
        }
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

    let pool = AsyncComputeTaskPool::get();

    // TODO Replace `sorted_unstable_by_key` with `k_smallest_by` when it becomes
    // available. It'll be much more efficient than sorting the entire list and
    // grabbing only 2 or 3.
    let tasks = pending_tasks
        .iter()
        .map(|q| {
            let mut priority = f32::INFINITY;

            for (transform, anchor) in anchors.iter() {
                let pos = transform.translation.as_ivec3() >> 4;
                let anchor_priority = anchor.get_priority(pos, q.1.chunk_coords());
                priority = f32::min(priority, anchor_priority);
            }

            (q, OrderedFloat(priority))
        })
        .sorted_unstable_by_key(|q| q.1)
        .take(available_slots as usize)
        .map(|(q, _)| q);

    for (chunk_id, pending_task) in tasks {
        let world_id = pending_task.world_id();
        let chunk_coords = pending_task.chunk_coords();

        let generator = generators.get(world_id).ok().map(|g| g.generator());

        match generator {
            Some(gen) => {
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
    chunks: VoxelQuery<Entity, With<VoxelStorage<T>>>,
    mut load_chunk_tasks: Query<(Entity, &mut LoadChunkTask<T>, &VoxelChunk)>,
    mut commands: Commands,
) {
    for (chunk_id, mut task, chunk_meta) in load_chunk_tasks.iter_mut() {
        if let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) {
            commands
                .entity(chunk_id)
                .remove::<LoadChunkTask<T>>()
                .insert(chunk_data);

            #[cfg(feature = "meshing")]
            {
                let world_id = chunk_meta.world_id();
                let chunk_coords = chunk_meta.chunk_coords();
                for offset in Region::from_points(IVec3::NEG_ONE, IVec3::ONE).iter() {
                    if offset.x + offset.y + offset.z > 1 {
                        continue;
                    }

                    if let Ok(chunk_id) = chunks.get_chunk(world_id, chunk_coords + offset) {
                        commands.entity(chunk_id).insert(RemeshChunk);
                    }
                }
            }
        }
    }
}
