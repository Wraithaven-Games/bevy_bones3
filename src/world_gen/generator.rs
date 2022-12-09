//! This module defines the world generator trait and how it should be
//! implemented.

use std::marker::PhantomData;

use bevy::prelude::{Component, Entity, IVec3};

use crate::prelude::Region;
use crate::storage::{BlockData, VoxelWorldSlice};

/// A trait that handles the generation of block data when new chunks are
/// loaded.
pub trait WorldGenerator<T: BlockData>: Copy + Send + Sync + 'static {
    /// Generates a voxel world slice containing the block data to populate a
    /// newly generated chunk at the given chunk coordinates.
    fn generate_chunk(&self, chunk_coords: IVec3) -> VoxelWorldSlice<T>;
}

/// A component that holds a world generator.
#[derive(Debug, Clone, Component)]
pub struct WorldGeneratorHandler<T: BlockData, W: WorldGenerator<T>> {
    /// The entity of the world that this world generator is targeting.
    pub world: Option<Entity>,

    /// The world generator instance.
    pub generator: W,

    /// Phantom data for T.
    _phantom: PhantomData<T>,
}

impl<T: BlockData, W: WorldGenerator<T>> WorldGeneratorHandler<T, W> {
    /// Creates a new world generator handler instance for the given world and
    /// generator.
    pub fn new(world: Entity, generator: W) -> Self {
        Self {
            world: Some(world),
            generator,
            _phantom: PhantomData::default(),
        }
    }
}

/// An empty implementation of a world generator that always returns a voxel
/// world slice filled with the default value for T.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Default)]
pub struct EmptyWorldGenerator;

impl<T: BlockData> WorldGenerator<T> for EmptyWorldGenerator {
    fn generate_chunk(&self, chunk_coords: IVec3) -> VoxelWorldSlice<T> {
        VoxelWorldSlice::<T>::new(Region::CHUNK.shift(chunk_coords << 4))
    }
}
