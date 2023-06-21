//! A system parameter helper for executing voxel-specific commands.

use bevy::ecs::system::{Command, EntityCommands, SystemParam};
use bevy::prelude::*;

use super::VoxelQueryError;
use crate::storage::chunk_pointers::ChunkEntityPointers;
use crate::storage::{VoxelChunk, VoxelWorld};

/// A Bevy command queue helper for working with Voxel-based actions.
#[derive(SystemParam)]
pub struct VoxelCommands<'w, 's> {
    /// A mutable query of chunk entity pointers.
    chunk_pointers: Query<'w, 's, &'static ChunkEntityPointers, With<VoxelWorld>>,

    /// A list of all chunks within the Bevy entity list.
    all_chunks: Query<'w, 's, Entity, With<VoxelChunk>>,

    /// A reference to Bevy commands for triggering specific chunk commands.
    commands: Commands<'w, 's>,
}

impl<'w, 's, 'cmd_ref> VoxelCommands<'w, 's> {
    /// Gets whether or not the given world id is valid and queryable.
    ///
    /// This method will return false if the provided entity is not a valid
    /// voxel world or if the entity has already despawned.
    ///
    /// Note that this function does not chunk if there are any chunks within
    /// this voxel query or if the provided world  has any loaded chunks. This
    /// function simply checks to see if the world in question exists or
    /// not.
    pub fn has_world(&self, world_id: Entity) -> bool {
        self.chunk_pointers.contains(world_id)
    }

    /// Gets the command queue  for the voxel world with the given world id.
    ///
    /// This method will return an error is there is no valid voxel worlds with
    /// the given world id.
    pub fn get_world(
        &'cmd_ref mut self,
        world_id: Entity,
    ) -> Result<VoxelWorldCommands<'w, 's, 'cmd_ref>, VoxelQueryError> {
        if !self.has_world(world_id) {
            return Err(VoxelQueryError::WorldNotFound(world_id));
        }

        Ok(VoxelWorldCommands {
            voxel_commands: self,
            world_id,
        })
    }

    /// Gets a reference to the underlying Bevy commands queue.
    pub fn commands(&'cmd_ref mut self) -> &'cmd_ref mut Commands<'w, 's> {
        &mut self.commands
    }

    /// Spawns a new voxel world and attaches the given component bundle to it.
    /// A command queue handler for the newly generated voxel world object
    /// is returned for further editing.
    ///
    /// This function will also initialize the chunk pointer cache system for
    /// the voxel world as well.
    pub fn spawn_world<B>(&'cmd_ref mut self, bundle: B) -> VoxelWorldCommands<'w, 's, 'cmd_ref>
    where
        B: Bundle,
    {
        let world_id = self
            .commands
            .spawn((VoxelWorld, ChunkEntityPointers::default(), bundle))
            .id();

        VoxelWorldCommands {
            voxel_commands: self,
            world_id,
        }
    }
}

/// A Bevy command queue helper for working with Voxel world-based actions.
pub struct VoxelWorldCommands<'w, 's, 'cmd_ref> {
    /// A mutable reference to the overall voxel command queue.
    voxel_commands: &'cmd_ref mut VoxelCommands<'w, 's>,

    /// The entity id of the voxel world being handled.
    world_id: Entity,
}

impl<'w, 's, 'cmd_ref, 'chunk_ref> VoxelWorldCommands<'w, 's, 'cmd_ref> {
    /// Spawns a new chunk within the voxel world at the given chunk
    /// coordinates.
    ///
    /// The voxel chunk will spawn with the given component bundle attached.
    ///
    /// This method will return an error if there is already an existing chunk
    /// at the given chunk coordinates.
    pub fn spawn_chunk<B>(
        &'chunk_ref mut self,
        chunk_coords: IVec3,
        bundle: B,
    ) -> Result<VoxelChunkCommands<'w, 's, 'chunk_ref>, VoxelQueryError>
    where
        B: Bundle,
    {
        if self.get_chunk_id(chunk_coords).is_some() {
            return Err(VoxelQueryError::ChunkAlreadyExists(
                self.world_id,
                chunk_coords,
            ));
        }

        let chunk_id = self
            .voxel_commands
            .commands
            .spawn((VoxelChunk::new(self.world_id, chunk_coords), bundle))
            .set_parent(self.world_id)
            .id();

        self.voxel_commands.commands.add(UpdateChunkPointersAction {
            world_id: self.world_id,
            chunk_id: Some(chunk_id),
            chunk_coords,
        });

        Ok(VoxelChunkCommands {
            voxel_commands: self.voxel_commands,
            world_id: self.world_id,
            chunk_id,
            chunk_coords,
        })
    }

    /// Gets the chunk id of the given within this voxel world at the given
    /// chunk coordinates.
    ///
    /// This method will return None if there is no valid chunk at the given
    /// coordinates.
    ///
    /// Note that this method will only account for chunks that existed since
    /// the previous frame. Chunks that were spawned on the current frame,
    /// (before the command queue is executed) will always return None.
    pub fn get_chunk_id(&self, chunk_coords: IVec3) -> Option<Entity> {
        let pointers = self.voxel_commands.chunk_pointers.get(self.world_id).ok()?;

        let Some(chunk_id) = pointers.get_chunk_entity(chunk_coords) else {
            return None;
        };

        if !self.voxel_commands.all_chunks.contains(chunk_id) {
            return None;
        }

        Some(chunk_id)
    }

    /// Gets the voxel command queue for the chunk at the given voxel
    /// coordinates.
    ///
    /// Note that this method will only account for chunks that existed since
    /// the previous frame. Chunks that were spawned on the current frame,
    /// (before the command queue is executed) will always return None.
    pub fn get_chunk(
        &'chunk_ref mut self,
        chunk_coords: IVec3,
    ) -> Result<VoxelChunkCommands<'w, 's, 'chunk_ref>, VoxelQueryError> {
        let chunk_id = self
            .get_chunk_id(chunk_coords)
            .ok_or(VoxelQueryError::ChunkNotFound(self.world_id, chunk_coords))?;

        Ok(VoxelChunkCommands {
            voxel_commands: self.voxel_commands,
            world_id: self.world_id,
            chunk_id,
            chunk_coords,
        })
    }

    /// Gets the id of the voxel world being handled.
    pub fn id(&self) -> Entity {
        self.world_id
    }

    /// Gets the entity command queue for this voxel world object.
    pub fn as_entity_commands(self) -> EntityCommands<'w, 's, 'cmd_ref> {
        self.voxel_commands
            .commands
            .get_entity(self.world_id)
            .unwrap()
    }
}

/// A Bevy command queue helper for working with Voxel chunk-based actions.
pub struct VoxelChunkCommands<'world, 'state, 'cmd_ref> {
    /// A reference to the voxel command queue.
    voxel_commands: &'cmd_ref mut VoxelCommands<'world, 'state>,

    /// The id of the world the chunk is located in.
    world_id: Entity,

    /// The id of the chunk being handled.
    chunk_id: Entity,

    /// The coordinates of the chunk being handled.
    chunk_coords: IVec3,
}

impl<'world, 'state, 'cmd_ref> VoxelChunkCommands<'world, 'state, 'cmd_ref> {
    /// Despawns this chunk and all child entities attached to it, recursively.
    ///
    /// This method will also update the internal chunk pointer cache of the
    /// voxel world to reflect the changes.
    pub fn despawn(self) {
        self.voxel_commands
            .commands
            .entity(self.chunk_id)
            .despawn_recursive();

        self.voxel_commands.commands.add(UpdateChunkPointersAction {
            world_id:     self.world_id,
            chunk_id:     None,
            chunk_coords: self.chunk_coords,
        })
    }

    /// Gets the entity command queue for this voxel chunk object.
    pub fn as_entity_commands(self) -> EntityCommands<'world, 'state, 'cmd_ref> {
        self.voxel_commands
            .commands
            .get_entity(self.chunk_id)
            .unwrap()
    }

    /// Gets the voxel world command queue for the world that this chunk is in.
    pub fn as_world_commands(self) -> VoxelWorldCommands<'world, 'state, 'cmd_ref> {
        self.voxel_commands.get_world(self.world_id).unwrap()
    }

    /// Gets the id of the world that the chunk being handled is apart of.
    pub fn world_id(&self) -> Entity {
        self.world_id
    }

    /// Gets the id of the chunk being handled.
    pub fn id(&self) -> Entity {
        self.chunk_id
    }

    /// Gets the coordinates of the chunk being handled.
    pub fn chunk_coords(&self) -> IVec3 {
        self.chunk_coords
    }
}

/// A Bevy command that updates the internal chunk pointer cache for a voxel
/// world to indicate that a new chunk has been created or destroyed.
struct UpdateChunkPointersAction {
    /// The id of the world that is being edited.
    world_id: Entity,

    /// The new id of the voxel chunk, if it exists.
    chunk_id: Option<Entity>,

    /// The coordinates of the chunk within the world.
    chunk_coords: IVec3,
}

impl Command for UpdateChunkPointersAction {
    fn write(self, world: &mut World) {
        let mut pointers = world.get_mut::<ChunkEntityPointers>(self.world_id).unwrap();

        if pointers.get_chunk_entity(self.chunk_coords).is_some() && self.chunk_id.is_some() {
            panic!(
                "Tried to spawn chunk at {}, in world {:?}, but it already exists!",
                self.chunk_coords, self.world_id
            )
        };

        pointers.set_chunk_entity(self.chunk_coords, self.chunk_id);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build_world() {
        let mut app = App::new();

        fn init(mut commands: VoxelCommands) {
            commands
                .spawn_world(())
                .spawn_chunk(IVec3::new(13, 15, 17), ())
                .unwrap();
        }
        Schedule::new().add_system(init).run(&mut app.world);

        fn validate(world_query: Query<Entity, With<VoxelWorld>>, mut commands: VoxelCommands) {
            let world_id = world_query.get_single().unwrap();
            commands
                .get_world(world_id)
                .unwrap()
                .get_chunk(IVec3::new(13, 15, 17))
                .unwrap();
        }
        Schedule::new().add_system(validate).run(&mut app.world);
    }

    #[test]
    #[should_panic(
        expected = "Tried to spawn chunk at [0, 0, 0], in world 0v0, but it already exists!"
    )]
    fn spawn_two_identical_chunks_same_frame() {
        let mut app = App::new();

        fn init(mut commands: VoxelCommands) {
            commands.spawn_world(());
        }
        Schedule::new().add_system(init).run(&mut app.world);

        fn a(world_query: Query<Entity, With<VoxelWorld>>, mut commands: VoxelCommands) {
            let world_id = world_query.get_single().unwrap();
            commands
                .get_world(world_id)
                .unwrap()
                .spawn_chunk(IVec3::ZERO, ())
                .unwrap();
        }

        fn b(world_query: Query<Entity, With<VoxelWorld>>, mut commands: VoxelCommands) {
            let world_id = world_query.get_single().unwrap();
            commands
                .get_world(world_id)
                .unwrap()
                .spawn_chunk(IVec3::ZERO, ())
                .unwrap();
        }

        Schedule::new()
            .add_system(a)
            .add_system(b)
            .run(&mut app.world);
    }
}
