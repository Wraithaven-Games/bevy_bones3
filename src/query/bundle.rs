//! The recommended, voxel query-safe bundle for initializing a new voxel world.

use bevy::prelude::*;

use super::ChunkEntityPointers;
use crate::prelude::VoxelWorld;

/// A component bundle for creating a new, query-safe voxel world.
#[derive(Bundle, Default)]
pub struct VoxelWorldBundle {
    /// The voxel world marker.
    world: VoxelWorld,

    /// The chunk entity pointers handler for the world.
    pointers: ChunkEntityPointers,
}
