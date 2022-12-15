//! The Bevy system parameter value.

use bevy::ecs::query::{QueryItem, QueryIter, ROQueryItem, ReadOnlyWorldQuery, WorldQuery};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::sector::ChunkEntityPointers;
use super::VoxelQueryError;
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

    /// A list of all chunks within the Bevy entity list.
    all_chunks: Query<'w, 's, Entity, With<VoxelChunk>>,
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
            .and_then(|e| self.all_chunks.get(e).ok())
            .ok_or(VoxelQueryError::ChunkNotFound(world_id, chunk_coords))
    }

    /// Queries the chunk at the given coordinates within the specified world.
    ///
    /// This function may return an error when querying chunks that have been
    /// despawned or otherwise do not exist, or if the chunk is simply not part
    /// of the requested world query.
    ///
    /// To find a chunk entity at a specific location, regardless of the query
    /// filter, use [`find_chunk`] instead.
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
    /// despawned or otherwise do not exist, or if the chunk is simply not part
    /// of the requested world query.
    ///
    /// To find a chunk entity at a specific location, regardless of the query
    /// filter, use [`find_chunk`] instead.
    pub fn get_chunk_mut(
        &mut self,
        world_id: Entity,
        chunk_coords: IVec3,
    ) -> Result<QueryItem<'_, Q>, VoxelQueryError> {
        self.chunks
            .get_mut(self.find_chunk(world_id, chunk_coords)?)
            .map_err(VoxelQueryError::QueryError)
    }
}
