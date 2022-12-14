//! The Bevy system parameter value.

use bevy::ecs::query::{
    QueryEntityError,
    QueryItem,
    QueryIter,
    ROQueryItem,
    ReadOnlyWorldQuery,
    WorldQuery,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::ChunkEntityPointers;
use crate::storage::{VoxelChunk, VoxelWorld};

/// A system parameter designed for quickly querying and reading from voxel
/// worlds.
///
/// In order to use VoxelChunkQueries, all affected worlds must include the
/// ChunkEntityPointers component, and it must be correctly up to date. It is
/// highly recommended to only spawn and despawn chunks using [`VoxelQueryMut`]
#[derive(SystemParam)]
pub struct VoxelQuery<'w, 's, Q, F = ()>
where
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// A readonly query of chunk entity pointers.
    chunk_pointers: Query<'w, 's, (Entity, &'static ChunkEntityPointers), With<VoxelWorld>>,

    /// A standard query of voxel chunks.
    chunks: Query<'w, 's, Q, (With<VoxelChunk>, F)>,

    /// A reference to Bevy commands for triggering specific chunk commands.
    commands: Commands<'w, 's>,
}

impl<'w, 's, Q, F> VoxelQuery<'w, 's, Q, F>
where
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// Gets an iterator over all valid voxel worlds.
    pub fn worlds_iter(&'w self) -> impl Iterator<Item = Entity> + '_ {
        self.chunk_pointers.iter().map(|(entity, _)| entity)
    }

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

    /// Gets a readonly iterator over all chunks for the given world query.
    pub fn iter(&self) -> QueryIter<'_, 's, Q::ReadOnly, (With<VoxelChunk>, F::ReadOnly)> {
        self.chunks.iter()
    }

    /// Gets a mutable iterator over all chunks for the given world query.
    pub fn iter_mut(&mut self) -> QueryIter<'_, 's, Q, (With<VoxelChunk>, F)> {
        self.chunks.iter_mut()
    }

    /// Gets the entity id of the chunk at the given coordinates within the
    /// indicated world.
    ///
    /// Returns an error if the world id is not valid or could not be found, or
    /// if the world does not contain any chunks with the given chunk
    /// coordinates.
    ///
    /// Note that the returned Entity id may still point to an entity that has
    /// already been despawned.
    pub fn find_chunk(
        &self,
        world_id: Entity,
        chunk_coords: IVec3,
    ) -> Result<Entity, VoxelQueryError> {
        let pointers = self
            .chunk_pointers
            .get(world_id)
            .map_err(|_| VoxelQueryError::WorldNotFound(world_id))?
            .1;

        pointers
            .get_chunk_entity(chunk_coords)
            .ok_or(VoxelQueryError::ChunkNotFound(world_id, chunk_coords))
    }

    /// Queries the chunk at the given coordinates within the specified world.
    ///
    /// This function may return an error when querying chunks that have been
    /// despawned or otherwise do not exist.
    pub fn get_chunk(
        &self,
        world_id: Entity,
        chunk_coords: IVec3,
    ) -> Result<ROQueryItem<'_, Q>, VoxelQueryError> {
        self.chunks
            .get(self.find_chunk(world_id, chunk_coords)?)
            .map_err(VoxelQueryError::QueryError)
    }

    /// Queries the chunk at the given coordinates within the specified world.
    ///
    /// This function may return an error when querying chunks that have been
    /// despawned or otherwise do not exist.
    pub fn get_chunk_mut(
        &mut self,
        world_id: Entity,
        chunk_coords: IVec3,
    ) -> Result<QueryItem<'_, Q>, VoxelQueryError> {
        self.chunks
            .get_mut(self.find_chunk(world_id, chunk_coords)?)
            .map_err(VoxelQueryError::QueryError)
    }

    /// Triggers the target chunk to be remeshed.
    #[cfg(feature = "meshing")]
    pub fn remesh_chunk(
        &mut self,
        world_id: Entity,
        chunk_coords: IVec3,
    ) -> Result<(), VoxelQueryError> {
        use crate::prelude::RemeshChunk;

        let chunk_id = self.find_chunk(world_id, chunk_coords)?;
        let mut c = self
            .commands
            .get_entity(chunk_id)
            .ok_or(VoxelQueryError::ChunkNotFound(world_id, chunk_coords))?;

        c.insert(RemeshChunk);
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
}

/// An error type that is thrown while handling voxel queries.
#[derive(Debug)]
pub enum VoxelQueryError {
    /// Thrown when attempting to read from an invalid or non-existent world.
    WorldNotFound(Entity),

    /// Throw when there is no chunk located at the given chunk coordinates
    /// within a specific world.
    ChunkNotFound(Entity, IVec3),

    /// A standard Bevy query error.
    QueryError(QueryEntityError),
}
