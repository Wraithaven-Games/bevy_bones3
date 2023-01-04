//! Errors that can be triggered when working with voxel queries and commands.

use bevy::ecs::query::QueryEntityError;
use bevy::prelude::*;
use thiserror::Error;

/// An error type that is thrown while handling voxel queries.
#[derive(Debug, Error)]
pub enum VoxelQueryError {
    /// Thrown when attempting to read from an invalid or non-existent world.
    #[error("Cannot find world with id {0:?}")]
    WorldNotFound(Entity),

    /// Throw when there is no chunk located at the given chunk coordinates
    /// within a specific world.
    #[error("Cannot find chunk at {1} within the world {0:?}")]
    ChunkNotFound(Entity, IVec3),

    /// Thrown when attempting to spawn a new chunk that already exists.
    #[error("There is already a chunk located at {1} within the world {0:?}")]
    ChunkAlreadyExists(Entity, IVec3),

    /// A standard Bevy query error.
    #[error("Failed to query chunks")]
    QueryError(#[from] QueryEntityError),
}
