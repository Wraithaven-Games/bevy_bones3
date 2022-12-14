//! Useful extension traits to add voxel world/chunk spawning and despawning to
//! Bevy commands.

use bevy::ecs::system::{Command, EntityCommands};
use bevy::prelude::*;

use super::ChunkEntityPointers;
use crate::storage::{BlockData, VoxelChunk, VoxelWorld};

/// An extension trait for adding voxel world and voxel chunk support for Bevy
/// commands.
pub trait VoxelCommands<'w, 's> {
    /// Gets the voxel world with the specified world id. (Entity)
    ///
    /// If there is no voxel world with the given world id then None is
    /// returned.
    fn voxel_world<'a>(&'a mut self, world_id: Entity) -> Option<VoxelWorldCommands<'w, 's, 'a>>;

    /// Spawns a new voxel world.
    fn spawn_world<'a, T, B>(&'a mut self, bundle: B) -> VoxelWorldCommands<'w, 's, 'a>
    where
        T: BlockData,
        B: Bundle;
}

impl<'w, 's> VoxelCommands<'w, 's> for Commands<'w, 's> {
    fn voxel_world<'a>(&'a mut self, world_id: Entity) -> Option<VoxelWorldCommands<'w, 's, 'a>> {
        self.get_entity(world_id)?;

        Some(VoxelWorldCommands {
            world_id,
            commands: self,
        })
    }

    fn spawn_world<'a, T, B>(&'a mut self, bundle: B) -> VoxelWorldCommands<'w, 's, 'a>
    where
        T: BlockData,
        B: Bundle,
    {
        let world_id = self
            .spawn((VoxelWorld, ChunkEntityPointers::default()))
            .insert(bundle)
            .id();

        VoxelWorldCommands::<'w, 's, 'a> {
            world_id,
            commands: self,
        }
    }
}

/// A set of commands for handling voxel worlds.
pub struct VoxelWorldCommands<'w, 's, 'a> {
    /// The world id of the voxel world.
    world_id: Entity,

    /// The Bevy command queue.
    commands: &'a mut Commands<'w, 's>,
}

impl<'w, 's, 'a> VoxelWorldCommands<'w, 's, 'a> {
    /// Gets the id of the voxel world.
    pub fn id(&self) -> Entity {
        self.world_id
    }

    /// Gets the standard Bevy entity commands handler for this voxel world
    /// entity.
    pub fn entity_commands(&'a mut self) -> EntityCommands<'w, 's, 'a> {
        self.commands.entity(self.world_id)
    }

    /// Despawns this world and all child chunks for it.
    ///
    /// This is just a shortcut method for calling
    /// `commands.entity_commands().despawn_recursive()`
    pub fn despawn(&'a mut self) {
        self.entity_commands().despawn_recursive()
    }

    /// Spawns a new chunk for the target voxel world at the given chunk
    /// coordinates.
    pub fn spawn_chunk<B>(
        &'a mut self,
        chunk_coords: IVec3,
        bundle: B,
    ) -> EntityCommands<'w, 's, 'a>
    where
        B: Bundle,
    {
        let chunk_id = self
            .commands
            .spawn(VoxelChunk::new(self.world_id, chunk_coords))
            .insert(bundle)
            .set_parent(self.world_id)
            .id();

        self.commands.add(UpdateChunkEntityPointerAction {
            world_id: self.world_id,
            chunk_id: Some(chunk_id),
            chunk_coords,
        });

        self.commands.entity(chunk_id)
    }
}

/// A Bevy command action that updates the chunk pointer cache for a voxel world
/// to indicate a new entity pointer for a given set of chunk coordinates.
struct UpdateChunkEntityPointerAction {
    /// The id of the world being manipulated.
    world_id: Entity,

    /// The new chunk id to store in the cache.
    chunk_id: Option<Entity>,

    /// The coordinates of the chunk to modify.
    chunk_coords: IVec3,
}

impl Command for UpdateChunkEntityPointerAction {
    fn write(self, world: &mut World) {
        let mut chunk_pointers = world.get_mut::<ChunkEntityPointers>(self.world_id).unwrap();
        if chunk_pointers.get_chunk_entity(self.chunk_coords).is_none() {
            chunk_pointers.set_chunk_entity(self.chunk_coords, self.chunk_id);
        } else {
            panic!(
                "Attempted to create a chunk on top of an existing one at {}",
                self.chunk_coords
            );
        }
    }
}
