//! This module contains the components that may be used to generate chunk
//! meshes and interact with the remesh systems.

use bevy::prelude::*;

/// A temporary marker component that indicates that the target chunk needs to
/// be remeshed.
#[derive(Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct RemeshChunk;

/// An entity with this marker indicates that the entity exists only as a child
/// of a chunk to render it's physical mesh object.
#[derive(Component, Reflect)]
pub struct ChunkMesh;

/// This component, usually placed on the camera, is used to determine the
/// priority value for remeshing dirty chunks. If multiple chunks are awaiting a
/// remesh update, chunks that are closer in world space to this anchor are
/// prioritized over chunks that are further away.
#[derive(Component, Reflect)]
pub struct ChunkMeshCameraAnchor;
