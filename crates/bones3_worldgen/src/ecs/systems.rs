use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bones3_core::query::VoxelCommands;
use bones3_core::storage::{BlockData, VoxelChunk, VoxelStorage, VoxelWorld};
use bones3_core::util::anchor::{ChunkAnchor, ChunkAnchorRecipient};
#[cfg(feature = "meshing")]
use bones3_remesh::{ecs::components::RemeshChunk, query::VoxelRemeshCommands};
use futures_lite::future;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

use super::components::{LoadChunkTask, PendingLoadChunkTask, WorldGeneratorHandler};
use crate::WorldGenAnchor;

pub(crate) fn create_chunk_entities(
    anchors: Query<&ChunkAnchor<WorldGenAnchor>>,
    mut commands: VoxelCommands,
) {
    for anchor in anchors.iter() {
        let Ok(mut world_commands) = commands.get_world(anchor.world_id) else {
            continue;
        };

        let Some(region) = anchor.get_region() else {
            continue;
        };

        for chunk_coords in region.into_iter() {
            let chunk_pos = chunk_coords.as_vec3() * 16.0;

            world_commands
                .spawn_chunk(
                    chunk_coords,
                    SpatialBundle {
                        transform: Transform::from_translation(chunk_pos),
                        ..default()
                    },
                )
                // Ignore the result of spawn chunk.
                // If the chunk already exists, an error is thrown and we can safely ignore it.
                // If no error is returned, a new chunk is correctly created instead.
                .ok();
        }
    }
}

pub(crate) fn unload_chunks(
    chunks: Query<(&ChunkAnchorRecipient<WorldGenAnchor>, &VoxelChunk)>,
    mut commands: VoxelCommands,
) {
    for (anchor_recipient, chunk_meta) in chunks.iter() {
        if anchor_recipient.priority.is_none() {
            let Ok(mut world_commands) = commands.get_world(chunk_meta.world_id()) else {
                continue;
            };

            let Ok(chunk_commands) = world_commands.get_chunk(chunk_meta.chunk_coords()) else {
                continue;
            };

            chunk_commands.despawn();
        }
    }
}

pub(crate) fn queue_chunks<T>(
    chunks: Query<
        Entity,
        (
            With<VoxelChunk>,
            Without<VoxelStorage<T>>,
            Without<PendingLoadChunkTask>,
            Without<LoadChunkTask<T>>,
        ),
    >,
    mut commands: Commands,
) where
    T: BlockData,
{
    for chunk_id in chunks.iter() {
        commands.entity(chunk_id).insert(PendingLoadChunkTask);
    }
}

/// Moves queued chunk loading tasks to an active async chunk loading task.
pub(crate) fn push_chunk_async_queue<T>(
    active_tasks: Query<(Entity, &LoadChunkTask<T>)>,
    chunks: Query<
        (&ChunkAnchorRecipient<WorldGenAnchor>, &VoxelChunk, Entity),
        With<PendingLoadChunkTask>,
    >,
    generators: Query<&WorldGeneratorHandler<T>, With<VoxelWorld>>,
    mut commands: Commands,
) where
    T: BlockData,
{
    // TODO Move this value to a resource.
    /// The maximum number of async world generations tasks that can exist at
    /// once.
    const MAX_TASKS: i32 = 3;

    let available_slots = MAX_TASKS - active_tasks.iter().len() as i32;
    if available_slots <= 0 {
        return;
    }

    let pool = AsyncComputeTaskPool::get();
    for (chunk_coords, chunk_id, world_id) in get_max_chunks(&chunks, available_slots as usize) {
        match generators.get(world_id).ok().map(|g| g.generator()) {
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
pub(crate) fn finish_chunk_loading<T: BlockData>(
    mut load_chunk_tasks: Query<(Entity, &mut LoadChunkTask<T>, &VoxelChunk)>,
    mut commands: VoxelCommands,
) {
    for (chunk_id, mut task, chunk_meta) in load_chunk_tasks.iter_mut() {
        let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        let mut c = commands.commands().entity(chunk_id);
        c.remove::<LoadChunkTask<T>>().insert(chunk_data);

        #[cfg(feature = "meshing")]
        {
            c.insert(RemeshChunk);
            commands
                .get_world(chunk_meta.world_id())
                .unwrap()
                .get_chunk(chunk_meta.chunk_coords())
                .unwrap()
                .remesh_chunk_neighbors();
        }
    }
}

fn get_max_chunks(
    chunks: &Query<
        (&ChunkAnchorRecipient<WorldGenAnchor>, &VoxelChunk, Entity),
        With<PendingLoadChunkTask>,
    >,
    max_chunks: usize,
) -> impl Iterator<Item = (IVec3, Entity, Entity)> {
    let mut queue = PriorityQueue::new();

    for (anchor_recipient, chunk_meta, chunk_id) in chunks.iter() {
        let Some(priority) = anchor_recipient.priority else {
            continue;
        };

        queue.push(
            (chunk_meta.chunk_coords(), chunk_id, chunk_meta.world_id()),
            OrderedFloat::from(priority),
        );
    }

    queue.into_sorted_iter().take(max_chunks).map(|(e, _)| e)
}
