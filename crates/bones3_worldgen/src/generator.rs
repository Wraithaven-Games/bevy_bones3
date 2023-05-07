//! This module defines the world generator trait and how it should be
//! implemented.

use std::sync::Arc;

use bevy::prelude::*;
use bones3_core::prelude::{BlockData, VoxelStorage};

/// A trait that handles the generation of block data when new chunks are
/// loaded.
pub trait WorldGenerator<T>
where
    T: BlockData,
    Self: Send + Sync,
{
    /// Generates a voxel world slice containing the block data to populate a
    /// newly generated chunk at the given chunk coordinates.
    fn generate_chunk(&self, chunk_coords: IVec3) -> VoxelStorage<T>;
}

/// A component wrapper for storing a WorldGenerator object.
#[derive(Component, Reflect)]
pub struct WorldGeneratorHandler<T>(#[reflect(ignore)] Arc<dyn WorldGenerator<T>>)
where
    T: BlockData;

impl<T> WorldGeneratorHandler<T>
where
    T: BlockData,
{
    /// Creates a new WorldGeneratorHandler instance.
    pub fn from<G>(generator: G) -> Self
    where
        G: WorldGenerator<T> + 'static,
    {
        Self(Arc::new(generator))
    }

    /// Gets a reference to the world generator instance.
    pub fn generator(&self) -> Arc<dyn WorldGenerator<T>> {
        self.0.clone()
    }
}
