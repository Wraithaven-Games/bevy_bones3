//! This module acts as a set of cleanup handlers for ensuring that all newly
//! generated chunk correctly inherit required components.

use bevy::prelude::*;
use bones3_core::prelude::{VoxelChunk, VoxelWorld};

/// This system checks for newly created voxel chunks that are children of
/// world with transforms. As they are discovered, Spacial bundles are applied
/// to them to ensure a clean hierarchy.
pub fn setup_chunk_transforms(
    worlds: Query<(), (With<Transform>, With<VoxelWorld>)>,
    new_chunks: Query<(Entity, &VoxelChunk), Added<VoxelChunk>>,
    mut commands: Commands,
) {
    for (chunk_id, chunk_meta) in new_chunks.iter() {
        let world_id = chunk_meta.world_id();
        if !worlds.contains(world_id) {
            continue;
        }

        let pos = chunk_meta.chunk_coords().as_vec3() * 16.0;
        commands.entity(chunk_id).insert(SpatialBundle {
            transform: Transform::from_translation(pos),
            ..default()
        });
    }
}
