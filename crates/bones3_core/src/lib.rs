//! This cargo crate implements the core functionality for Bones Cubed. This
//! includes the voxel storage capabilities, world data structures, voxel
//! queries and system parameters, and similar features.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::invalid_html_tags)]
#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

use bevy::prelude::*;
use prelude::storage::chunk_pointers::ChunkEntityPointers;
use prelude::*;

pub mod math;
pub mod query;
pub mod storage;

/// Used to import common components and systems for Bones Cubed.
pub mod prelude {
    pub use super::math::*;
    pub use super::query::*;
    pub use super::storage::*;
    pub use super::*;
}

/// The core plugin for Bones Cubed.
#[derive(Default)]
pub struct Bones3CorePlugin<T>
where
    T: BlockData,
{
    /// Phantom data for T.
    _phantom: PhantomData<T>,
}

impl<T> Plugin for Bones3CorePlugin<T>
where
    T: BlockData,
{
    fn build(&self, app: &mut App) {
        app.register_type::<VoxelWorld>()
            .register_type::<VoxelChunk>()
            .register_type::<VoxelStorage<T>>()
            .register_type::<ChunkEntityPointers>();
    }
}
