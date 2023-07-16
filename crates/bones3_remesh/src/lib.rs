//! This crate is designed to add chunk mesh generation support for Bones Cubed.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::invalid_html_tags)]
#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

use bevy::prelude::*;
use bones3_core::storage::BlockData;
use bones3_core::util::anchor::ChunkAnchorPlugin;
use ecs::resources::ChunkMaterialList;

use crate::ecs::components::*;
use crate::ecs::systems::*;
use crate::mesh::block_model::BlockShape;

pub mod ecs;
pub mod mesh;
pub mod query;
pub mod vertex_data;

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
            .register_type::<RemeshChunkTask<T>>()
            .insert_resource(ChunkMaterialList::default())
            .add_plugins(ChunkAnchorPlugin::<RemeshAnchor>::default())
            .add_systems(PostUpdate, remesh_dirty_chunks::<T>);
    }
}

/// The type definition to use for the `ChunkAnchorPlugin`.
#[derive(Default, Reflect)]
pub struct RemeshAnchor;

/// The system set in which all chunks are remeshed.
#[derive(Debug, SystemSet, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RemeshSet;
