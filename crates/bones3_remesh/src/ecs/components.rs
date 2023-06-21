//! This module contains the components that may be used to generate chunk
//! meshes and interact with the remesh systems.

use bevy::prelude::*;
use bevy::tasks::Task;
use bones3_core::storage::{BlockData, VoxelStorage};

/// A temporary marker component that indicates that the target chunk needs to
/// be remeshed.
#[derive(Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct RemeshChunk;

/// An entity with this marker indicates that the entity exists only as a child
/// of a chunk to render it's physical mesh object.
#[derive(Component, Reflect)]
pub struct ChunkMesh;

/// this component represents an active chunk that is currently being remeshed.
#[derive(Debug, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct RemeshChunkTask<T: BlockData>(#[reflect(ignore)] pub(crate) Task<VoxelStorage<T>>);
