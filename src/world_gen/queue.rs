//! The chunk loading queue to determine which chunks need to be loaded and how
//! soon they need to be loaded.

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

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

/// This function will trigger all of the chunks that are requested by chunk
/// anchors to begin loading.
pub fn load_chunks_async<T: BlockData, W: WorldGenerator<T>>(
    generators: Query<&WorldGeneratorHandler<T, W>>,
    mut worlds: Query<&mut VoxelWorld<T>>,
    mut anchors: Query<(&Transform, &mut ChunkAnchor)>,
    mut commands: Commands,
) {
    let mut pool: Option<&AsyncComputeTaskPool> = None;

    for (transform, mut anchor) in anchors.iter_mut() {
        let Some(world_entity) = anchor.get_world() else {
            continue;
        };

        let Ok(mut world) = worlds.get_mut(world_entity) else {
            warn!("Chunk anchor points to world that doesn't exist");
            anchor.set_world(None);
            continue;
        };

        let generator: Option<W> = generators
            .iter()
            .find(|g| g.world == Some(world_entity))
            .map(|g| g.generator);

        let anchor_pos = transform.translation.as_ivec3() >> 4;
        for chunk_coords in anchor.iter(anchor_pos) {
            if world.get_chunk_load_state(chunk_coords) != ChunkLoadState::Unloaded {
                continue;
            }

            if let Err(err) = world.prepare_chunk(chunk_coords) {
                warn!("Failed to prepare chunk for chunk loading\nErr: {}", err);
                continue;
            };

            if pool.is_none() {
                pool = Some(AsyncComputeTaskPool::get());
            }

            let chunk_region = Region::CHUNK.shift(chunk_coords << 4);
            let task = pool.unwrap().spawn(async move {
                if let Some(generator) = generator {
                    generator.generate_chunk(chunk_coords)
                } else {
                    VoxelWorldSlice::<T>::new(chunk_region)
                }
            });

            commands.spawn(LoadChunkTask {
                task,
                world: world_entity,
                chunk_coords,
            });
        }
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
