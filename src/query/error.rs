//! Errors that can be triggered when working with voxel queries and commands.

use bevy::ecs::query::QueryEntityError;
use bevy::prelude::*;

/// An error type that is thrown while handling voxel queries.
#[derive(Debug)]
pub enum VoxelQueryError {
    /// Thrown when attempting to read from an invalid or non-existent world.
    WorldNotFound(Entity),

    /// Throw when there is no chunk located at the given chunk coordinates
    /// within a specific world.
    ChunkNotFound(Entity, IVec3),

    /// This error is thrown when a chunk entity most likely exists, but is not
    /// currently available. This can happen when attempting to read a chunk
    /// that has been spawned, but won't actually be created until the next
    /// frame.
    ///
    /// This can also be triggered when manually despawning chunks and
    /// corrupting the chunk entity pointer cache, leading pointers to dead
    /// chunks to exist.
    ChunkNotAvailable(Entity, IVec3),

    /// Thrown when attempting to spawn a new chunk that already exists.
    ChunkAlreadyExists(Entity, IVec3),

    /// A standard Bevy query error.
    QueryError(QueryEntityError),
}
