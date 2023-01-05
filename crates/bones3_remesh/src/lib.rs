//! This crate is designed to add chunk mesh generation support for Bones Cubed.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::invalid_html_tags)]

use std::marker::PhantomData;

use bevy::prelude::*;
use bones3_core::storage::BlockData;

use crate::ecs::components::*;
use crate::ecs::systems::*;
use crate::mesh::block_model::BlockShape;

pub mod ecs;
pub mod mesh;
pub mod vertex_data;

/// Used to import common components and systems for Bones Cubed.
pub mod prelude {
    pub use super::ecs::components::*;
    pub use super::mesh::block_model::*;
    pub use super::mesh::error::*;
    pub use super::vertex_data::*;
    pub use super::*;
}

/// The remesh plugin for Bones Cubed.
#[derive(Default)]
pub struct Bones3RemeshPlugin<T>
where
    T: BlockData + BlockShape,
{
    /// Phantom data for T.
    _phantom: PhantomData<T>,
}

impl<T> Plugin for Bones3RemeshPlugin<T>
where
    T: BlockData + BlockShape,
{
    fn build(&self, app: &mut App) {
        app.register_type::<RemeshChunk>()
            .register_type::<ChunkMesh>()
            .register_type::<ChunkMeshCameraAnchor>()
            .add_system(remesh_dirty_chunks::<T>);
    }
}
