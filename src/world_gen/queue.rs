//! The chunk loading queue to determine which chunks need to be loaded and how
//! soon they need to be loaded.

use std::cmp::Ordering;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use itertools::Itertools;
use ordered_float::OrderedFloat;

use super::generator::WorldGeneratorHandler;
use super::{ChunkAnchor, WorldGenerator};
use crate::prelude::Region;
use crate::storage::{
    BlockData,
    ChunkLoadEvent,
    ChunkLoadState,
    ChunkStorage,
    VoxelWorld,
    VoxelWorldSlice,
};

/// A container for chunks that are actively being loading in an async thread
/// pool.
#[derive(Component)]
pub struct LoadChunkTask<T: BlockData> {
    /// The async chunk loading task.
    task: Task<VoxelWorldSlice<T>>,

    /// The world that this chunk is being generated for.
    world: Entity,

    /// The coordinates of the chunk being generated.
    chunk_coords: IVec3,
}

/// Defines an async chunk loading task that is queued but has not yet been
/// pushed to the compute task pool.
#[derive(Debug, Component, PartialEq, Eq)]
#[component(storage = "SparseSet")]
pub struct PendingLoadChunkTask {
    /// The world that this chunk is being generated for.
    world: Entity,

    /// The coordinates of the chunk being generated.
    chunk_coords: IVec3,

    /// The priority level of the chunk.
    priority: OrderedFloat<f32>,
}

impl Ord for PendingLoadChunkTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for PendingLoadChunkTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

/// This function will trigger all of the chunks that are requested by chunk
/// anchors to begin loading.
pub fn load_chunks_async<T: BlockData, W: WorldGenerator<T>>(
    mut worlds: Query<&mut VoxelWorld<T>>,
    mut anchors: Query<(&Transform, &mut ChunkAnchor)>,
    mut commands: Commands,
) {
    for (transform, mut anchor) in anchors.iter_mut() {
        let Some(world_entity) = anchor.get_world() else {
            continue;
        };

        let Ok(mut world) = worlds.get_mut(world_entity) else {
            warn!("Chunk anchor points to world that doesn't exist");
            anchor.set_world(None);
            continue;
        };

        let anchor_pos = transform.translation.as_ivec3() >> 4;
        for (chunk_coords, priority) in anchor.iter(anchor_pos) {
            if world.get_chunk_load_state(chunk_coords) != ChunkLoadState::Unloaded {
                continue;
            }

            if let Err(err) = world.prepare_chunk(chunk_coords) {
                warn!("Failed to prepare chunk for chunk loading\nErr: {}", err);
                continue;
            };

            commands.spawn(PendingLoadChunkTask {
                world: world_entity,
                chunk_coords,
                priority,
            });
        }
    }
}

/// Moves queued chunk loading tasks to an active async chunk loading task.
pub fn push_chunk_async_queue<T: BlockData, W: WorldGenerator<T>, const MAX_TASKS: u8>(
    active_tasks: Query<(Entity, &LoadChunkTask<T>)>,
    pending_tasks: Query<(Entity, &PendingLoadChunkTask)>,
    generators: Query<&WorldGeneratorHandler<T, W>>,
    mut commands: Commands,
) {
    let available_slots = MAX_TASKS as i32 - active_tasks.iter().len() as i32;
    if available_slots <= 0 {
        return;
    }

    let pool = AsyncComputeTaskPool::get();

    // TODO Replace with `iter().k_smallest_by(|a| a.1.priority)` when it becomes
    // available It'll be much more efficient than sorting the entire list and
    // grabbing only 2 or 3.
    let tasks = pending_tasks
        .iter()
        .sorted_unstable_by_key(|a| a.1.priority)
        .take(available_slots as usize);

    for (entity, pending_task) in tasks {
        let world = pending_task.world;
        let chunk_coords = pending_task.chunk_coords;
        let chunk_region = Region::CHUNK.shift(chunk_coords << 4);

        let generator: Option<W> = generators
            .iter()
            .find(|g| g.world == Some(world))
            .map(|g| g.generator);

        let task = pool.spawn(async move {
            if let Some(generator) = generator {
                generator.generate_chunk(chunk_coords)
            } else {
                VoxelWorldSlice::<T>::new(chunk_region)
            }
        });

        commands
            .entity(entity)
            .remove::<PendingLoadChunkTask>()
            .insert(LoadChunkTask {
                task,
                world,
                chunk_coords,
            });
    }
}

/// This system takes in all active async chunk loading tasks and, for each one
/// that is finished, push the results to the target voxel world.
pub fn finish_chunk_loading<T: BlockData>(
    mut commands: Commands,
    mut load_chunk_tasks: Query<(Entity, &mut LoadChunkTask<T>)>,
    mut worlds: Query<&mut VoxelWorld<T>>,
    mut chunk_load_ev: EventWriter<ChunkLoadEvent>,
) {
    for (entity, mut task) in load_chunk_tasks.iter_mut() {
        if let Some(world_slice) = future::block_on(future::poll_once(&mut task.task)) {
            commands.entity(entity).despawn();

            let Ok(mut world) = worlds.get_mut(task.world) else {
                // World has been unloaded
                continue;
            };

            world
                .init_chunk(task.chunk_coords)
                .fill_blocks(world_slice)
                .unwrap()
                .call_event(&mut chunk_load_ev, task.world)
                .unwrap();
        }
    }
}
