//! A system parameter helper for executing voxel-specific commands.

use bevy::ecs::system::{EntityCommands, SystemParam};
use bevy::prelude::*;

use super::sector::ChunkEntityPointers;
use super::VoxelQueryError;
use crate::prelude::{RemeshChunk, VoxelChunk, VoxelWorld};

/// A Bevy command queue helper for working with Voxel-based actions.
#[derive(SystemParam)]
pub struct VoxelCommands<'w, 's> {
    // TODO: Make this query readonly except when spawning/despawning chunks
    /// A mutable query of chunk entity pointers.
    chunk_pointers: Query<'w, 's, &'static mut ChunkEntityPointers, With<VoxelWorld>>,

    /// A list of all chunks within the Bevy entity list.
    all_chunks: Query<'w, 's, Entity, With<VoxelChunk>>,

    /// A reference to Bevy commands for triggering specific chunk commands.
    commands: Commands<'w, 's>,
}

impl<'w, 's, 'a> VoxelCommands<'w, 's> {
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

    /// Gets the entity id of the chunk at the given coordinates within the
    /// indicated world.
    ///
    /// Returns an error if the world id is not valid or could not be found, or
    /// if the world does not contain any chunks with the given chunk
    /// coordinates.
    pub fn find_chunk(
        &'a mut self,
        world_id: Entity,
        chunk_coords: IVec3,
        ignore_unavailable: bool,
    ) -> Result<EntityCommands<'w, 's, 'a>, VoxelQueryError> {
        let chunk_id = match self.get_pointers(world_id)?.get_chunk_entity(chunk_coords) {
            Some(chunk_id) => {
                if !ignore_unavailable {
                    self.all_chunks
                        .get(chunk_id)
                        .map_err(|_| VoxelQueryError::ChunkNotAvailable(world_id, chunk_coords))?;
                }
                chunk_id
            },
            None => {
                return Err(VoxelQueryError::ChunkNotFound(world_id, chunk_coords));
            },
        };

        Ok(self.commands.entity(chunk_id))
    }

    /// Gets a readonly reference to the chunk entity pointer handler for the
    /// given voxel world.
    ///
    /// This method returns an error if the world could not be found.
    fn get_pointers(&self, world_id: Entity) -> Result<&ChunkEntityPointers, VoxelQueryError> {
        self.chunk_pointers
            .get(world_id)
            .map_err(|_| VoxelQueryError::WorldNotFound(world_id))
    }

    /// Gets a mutable reference to the chunk entity pointer handler for the
    /// given voxel world.
    ///
    /// This method returns an error if the world could not be found.
    fn get_pointers_mut(
        &mut self,
        world_id: Entity,
    ) -> Result<Mut<'_, ChunkEntityPointers>, VoxelQueryError> {
        self.chunk_pointers
            .get_mut(world_id)
            .map_err(|_| VoxelQueryError::WorldNotFound(world_id))
    }

    /// Triggers the target chunk to be remeshed.
    #[cfg(feature = "meshing")]
    pub fn remesh_chunk(
        &mut self,
        world_id: Entity,
        chunk_coords: IVec3,
    ) -> Result<(), VoxelQueryError> {
        self.find_chunk(world_id, chunk_coords, true)?
            .insert(RemeshChunk);
        Ok(())
    }

    /// Triggers the target chunk and it's 6 surrounding neighboring chunks to
    /// be remeshed. The function will silently ignore any chunks that do not
    /// exist, except for the target chunk.
    #[cfg(feature = "meshing")]
    pub fn remesh_chunk_neighbors(
        &mut self,
        world_id: Entity,
        chunk_coords: IVec3,
    ) -> Result<(), VoxelQueryError> {
        self.remesh_chunk(world_id, chunk_coords)?;

        let _ = self.remesh_chunk(world_id, chunk_coords + IVec3::NEG_X);
        let _ = self.remesh_chunk(world_id, chunk_coords + IVec3::X);
        let _ = self.remesh_chunk(world_id, chunk_coords + IVec3::NEG_Y);
        let _ = self.remesh_chunk(world_id, chunk_coords + IVec3::Y);
        let _ = self.remesh_chunk(world_id, chunk_coords + IVec3::NEG_Z);
        let _ = self.remesh_chunk(world_id, chunk_coords + IVec3::Z);

        Ok(())
    }

    /// Triggers any chunks that are touching the specified block coordinates.
    #[cfg(feature = "meshing")]
    pub fn remesh_chunks_by_block(&mut self, world_id: Entity, block_coords: IVec3) {
        use itertools::Itertools;

        let chunks = vec![
            block_coords + IVec3::NEG_X,
            block_coords + IVec3::X,
            block_coords + IVec3::NEG_Y,
            block_coords + IVec3::Y,
            block_coords + IVec3::NEG_Z,
            block_coords + IVec3::Z,
        ]
        .iter()
        .map(|c| *c >> 4)
        .dedup()
        .collect::<Vec<IVec3>>();

        for chunk_coords in chunks.iter() {
            let _ = self.remesh_chunk(world_id, *chunk_coords);
        }
    }

    /// Spawns a new chunk within the indicated world at the specified chunk
    /// coordinates. The chunk will be spawned with the provided component
    /// bundle.
    ///
    /// This method will return the id of the newly generated chunk, or return
    /// an error if there is already a chunk at the given location within
    /// the world.
    pub fn spawn_chunk<B>(
        &'a mut self,
        world_id: Entity,
        chunk_coords: IVec3,
        bundle: B,
    ) -> Result<Entity, VoxelQueryError>
    where
        B: Bundle,
    {
        match self.find_chunk(world_id, chunk_coords, true) {
            Ok(_) => return Err(VoxelQueryError::ChunkAlreadyExists(world_id, chunk_coords)),
            Err(VoxelQueryError::ChunkNotFound(..)) => {},
            Err(err) => return Err(err),
        }

        let chunk_id = self
            .commands
            .spawn(VoxelChunk::new(world_id, chunk_coords))
            .insert(bundle)
            .set_parent(world_id)
            .id();

        self.get_pointers_mut(world_id)
            .unwrap()
            .set_chunk_entity(chunk_coords, Some(chunk_id));

        Ok(chunk_id)
    }

    /// Despawns, recursively, the chunk at the indicated chunk coordinates for
    /// the indicated voxel world.
    ///
    /// This function returns an error if the world could not be found or if the
    /// chunk does not exist.
    pub fn despawn_chunk(
        &'a mut self,
        world_id: Entity,
        chunk_coords: IVec3,
    ) -> Result<(), VoxelQueryError> {
        self.find_chunk(world_id, chunk_coords, true)?
            .despawn_recursive();

        self.get_pointers_mut(world_id)
            .unwrap()
            .set_chunk_entity(chunk_coords, None);

        Ok(())
    }

    /// Gets a reference to the underlying Bevy commands queue.
    pub fn commands(&'a mut self) -> &'a mut Commands<'w, 's> {
        &mut self.commands
    }
}
