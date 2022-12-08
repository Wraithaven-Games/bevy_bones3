//! This module contains the Bevy systems used to load and unload chunks based
//! on chunk anchor entities.

use bevy::prelude::*;

use super::cache::ChunkLoaderCache;
use super::ChunkAnchor;
use crate::prelude::ChunkLoadEvent;
use crate::storage::{BlockData, ChunkStorage, VoxelWorld};

/// This system will triggers new chunks to be loaded based on the current
/// locations of chunk anchors within the world.
pub fn load_chunks<const R: u8, T: BlockData>(
    anchors: Query<(&Transform, &ChunkAnchor)>,
    mut worlds: Query<&mut VoxelWorld<T>>,
    mut cache: Local<ChunkLoaderCache<R>>,
    mut chunk_load_ev: EventWriter<ChunkLoadEvent>,
) {
    for (transform, anchor) in anchors.iter() {
        if let Some(world_entity) = anchor.world {
            let mut world = worlds.get_mut(world_entity).unwrap();
            let center = transform.translation.as_ivec3() >> 4;
            let radius = anchor.radius as f32;

            cache.update_weighted_dir(anchor.weighted_dir);

            for chunk_coords in cache.iter(radius, center) {
                if !world.is_chunk_loaded(chunk_coords) {
                    world
                        .init_chunk(chunk_coords)
                        .call_event(&mut chunk_load_ev, world_entity)
                        .unwrap();
                    break;
                }
            }
        }
    }
}
