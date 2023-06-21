//! Contains extension functions for VoxelCommands.

use bevy::prelude::*;
use bones3_core::query::VoxelChunkCommands;

use crate::ecs::components::RemeshChunk;

/// An extension trait for VoxelChunkCommands that allow for a chunk to trigger
/// remeshing.
pub trait VoxelRemeshCommands {
    /// When called, this will mark the chunk as dirty by adding a remesh marker
    /// component to the chunk.
    fn remesh_chunk(self);

    /// When called, this will mark the chunk as dirty by adding a remesh marker
    /// component to the chunk, as well as all 6 major neighboring chunks.
    fn remesh_chunk_neighbors(self);

    /// When called, this will mark the chunk that the block is in as dirty by
    /// adding a remesh marker component to that chunk as well as any
    /// neighboring chunks that the given block touches.
    fn remesh_block(self, block_pos: IVec3);
}

impl<'w, 's, 'cmd_ref> VoxelRemeshCommands for VoxelChunkCommands<'w, 's, 'cmd_ref> {
    fn remesh_chunk(self) {
        self.as_entity_commands().insert(RemeshChunk);
    }

    fn remesh_chunk_neighbors(self) {
        let chunk_coords = self.chunk_coords();
        let mut world_commands = self.as_world_commands();

        world_commands
            .get_chunk(chunk_coords)
            .map_or((), |c| c.remesh_chunk());

        world_commands
            .get_chunk(chunk_coords + IVec3::X)
            .map_or((), |c| c.remesh_chunk());

        world_commands
            .get_chunk(chunk_coords + IVec3::Y)
            .map_or((), |c| c.remesh_chunk());

        world_commands
            .get_chunk(chunk_coords + IVec3::Z)
            .map_or((), |c| c.remesh_chunk());

        world_commands
            .get_chunk(chunk_coords - IVec3::X)
            .map_or((), |c| c.remesh_chunk());

        world_commands
            .get_chunk(chunk_coords - IVec3::Y)
            .map_or((), |c| c.remesh_chunk());

        world_commands
            .get_chunk(chunk_coords - IVec3::Z)
            .map_or((), |c| c.remesh_chunk());
    }

    fn remesh_block(self, block_pos: IVec3) {
        let block_pos = block_pos & 15;
        let chunk_coords = self.chunk_coords();
        let mut world_commands = self.as_world_commands();

        world_commands
            .get_chunk(chunk_coords)
            .map_or((), |c| c.remesh_chunk());

        if block_pos.x == 0 {
            world_commands
                .get_chunk(chunk_coords - IVec3::X)
                .map_or((), |c| c.remesh_chunk());
        }

        if block_pos.x == 15 {
            world_commands
                .get_chunk(chunk_coords + IVec3::X)
                .map_or((), |c| c.remesh_chunk());
        }

        if block_pos.y == 0 {
            world_commands
                .get_chunk(chunk_coords - IVec3::Y)
                .map_or((), |c| c.remesh_chunk());
        }

        if block_pos.y == 15 {
            world_commands
                .get_chunk(chunk_coords + IVec3::Y)
                .map_or((), |c| c.remesh_chunk());
        }

        if block_pos.z == 0 {
            world_commands
                .get_chunk(chunk_coords - IVec3::Z)
                .map_or((), |c| c.remesh_chunk());
        }

        if block_pos.z == 15 {
            world_commands
                .get_chunk(chunk_coords + IVec3::Z)
                .map_or((), |c| c.remesh_chunk());
        }
    }
}
