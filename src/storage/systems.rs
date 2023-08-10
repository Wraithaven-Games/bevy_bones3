//! This module defines systems used within this Bones3 storage sub-plugin.

use bevy::prelude::*;

use super::{SetBlockEvent, VoxelChunk, VoxelWorld};

/// This system is called whenever the set block event is triggered.
///
/// This system will modify the block data within the voxel chunk. It also
/// creates new chunks as needed.
pub(super) fn set_block(
    mut worlds: Query<&mut VoxelWorld>,
    mut chunks: Query<&mut VoxelChunk>,
    mut events: EventReader<SetBlockEvent>,
    mut commands: Commands,
) {
    for ev in events.iter() {
        let Ok(mut world) = worlds.get_mut(ev.world) else {
            warn!("Cannot set block for world {:?}, which does not exist.", &ev.world);
            continue;
        };

        if let Some(chunk_id) = world.get_chunk_id(ev.coords) {
            if let Ok(mut chunk) = chunks.get_mut(*chunk_id) {
                chunk.set(ev.coords, ev.block_id);
                continue;
            };
        };

        let mut storage = VoxelChunk::new(ev.coords);
        storage.set(ev.coords, ev.block_id);

        let chunk_id = commands.spawn(storage).id();
        world.update_chunk_id(ev.coords, chunk_id);
    }
}
