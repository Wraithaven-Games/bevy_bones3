use std::sync::Arc;

use bevy::prelude::*;
use bevy::tasks::Task;
use bones3_core::storage::{BlockData, VoxelStorage};

/// This component indicates that the chunk is currently being loaded in an
/// async task, and will have a voxel storage component replace this component
/// once it is done.
#[derive(Debug, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct LoadChunkTask<T: BlockData>(#[reflect(ignore)] pub(crate) Task<VoxelStorage<T>>);

/// A marker component that indicates that the target chunk is still waiting to
/// be loaded.
#[derive(Debug, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct PendingLoadChunkTask;

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
