//! The Bevy plugin for Bones3 for setting up all systems and components
//! required to use Bones3 in a Bevy app.

use std::marker::PhantomData;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

use crate::prelude::*;

/// The Bones Cubed plugin implementation.
///
/// This plugin group acts as a builder to allow for a more specific
/// customization of what parts of Bones Cubed are being used and what systems
/// to load in order to cleanly mesh with your existing game.
pub struct Bones3Plugin {
    /// The plugin group builder for this plugin group instance.
    builder: PluginGroupBuilder,
}

impl Bones3Plugin {
    /// Creates a new, empty Bones cubed plugin group instance.
    pub fn new() -> Self {
        Self {
            builder: PluginGroupBuilder::start::<Self>().add(CorePlugin),
        }
    }

    /// Adds support to Bones Cubed for the given block type.
    pub fn add_block_type<T>(mut self) -> Self
    where
        T: BlockData,
    {
        self.builder = self.builder.add(BlockTypePlugin::<T>::default());
        self
    }

    /// Adds chunk mesh support to generate meshes for blocks.
    ///
    /// This plugin requires the `meshing` feature of Bones Cubed to be enabled.
    #[cfg(feature = "meshing")]
    pub fn add_mesh_support(mut self) -> Self {
        self.builder = self.builder.add(MeshingPlugin);
        self
    }

    /// Adds support to Bones Cubed for mesh generation using the given block
    /// type.
    #[cfg(feature = "meshing")]
    pub fn add_mesh_block_type<T>(mut self) -> Self
    where
        T: BlockData + BlockShape,
    {
        self.builder = self.builder.add(MeshingBlockTypePlugin::<T>::default());
        self
    }

    /// Adds world generation support.
    ///
    /// This plugin requires the `world_gen` feature of Bones Cubed to be
    /// enabled.
    #[cfg(feature = "world_gen")]
    pub fn add_world_gen_support(mut self) -> Self {
        self.builder = self.builder.add(WorldGenPlugin);
        self
    }

    /// Adds support to use the given block type for world generation.
    ///
    /// This plugin requires the `world_gen` feature of Bones Cubed to be
    /// enabled.
    #[cfg(feature = "world_gen")]
    pub fn add_world_gen_block_type<T>(mut self) -> Self
    where
        T: BlockData,
    {
        self.builder = self.builder.add(WorldGenBlockTypePlugin::<T>::default());
        self
    }
}

impl Default for Bones3Plugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginGroup for Bones3Plugin {
    fn build(self) -> PluginGroupBuilder {
        self.builder
    }
}

/// The core plugin for enabling Bones Cubed functionality.
struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<VoxelWorld>()
            .register_type::<VoxelChunk>()
            .register_type::<ChunkEntityPointers>();
    }
}

/// Adds support to the core aspects of Bones Cubed to use the given block type.
#[derive(Default)]
struct BlockTypePlugin<T>(PhantomData<T>)
where
    T: BlockData;

impl<T> Plugin for BlockTypePlugin<T>
where
    T: BlockData,
{
    fn build(&self, app: &mut App) {
        app.register_type::<VoxelStorage<T>>();
    }
}

/// Adds support for chunk mesh generation.
///
/// This plugin requires the `meshing` feature of Bones Cubed to be enabled.
#[derive(Default)]
#[cfg(feature = "meshing")]
struct MeshingPlugin;

#[cfg(feature = "meshing")]
impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RemeshChunk>();
    }
}

/// Adds support for the given block type to the chunk remesh system.
///
/// This plugin requires the `meshing` feature of Bones Cubed to be enabled.
#[derive(Default)]
#[cfg(feature = "meshing")]
struct MeshingBlockTypePlugin<T>(PhantomData<T>)
where
    T: BlockData + BlockShape;

#[cfg(feature = "meshing")]
impl<T> Plugin for MeshingBlockTypePlugin<T>
where
    T: BlockData + BlockShape,
{
    fn build(&self, app: &mut App) {
        app.add_system(remesh_dirty_chunks::<T>);
    }
}

/// Adds support for world generation.
///
/// This plugin requires the `world_gen` feature of Bones Cubed to be enabled.
#[derive(Default)]
#[cfg(feature = "world_gen")]
struct WorldGenPlugin;

#[cfg(feature = "world_gen")]
impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ChunkAnchor>()
            .register_type::<PendingLoadChunkTask>()
            .add_system(setup_chunk_transforms);
    }
}

/// Adds support for world generation using the given block type.
///
/// This plugin requires the `world_gen` feature of Bones Cubed to be enabled.
#[derive(Default)]
#[cfg(feature = "world_gen")]
struct WorldGenBlockTypePlugin<T>(PhantomData<T>)
where
    T: BlockData;

#[cfg(feature = "world_gen")]
impl<T> Plugin for WorldGenBlockTypePlugin<T>
where
    T: BlockData,
{
    fn build(&self, app: &mut App) {
        app.register_type::<LoadChunkTask<T>>()
            .register_type::<WorldGeneratorHandler<T>>()
            .add_system(load_chunks_async::<T>)
            .add_system(push_chunk_async_queue::<T>)
            .add_system(finish_chunk_loading::<T>);
    }
}
