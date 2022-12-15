//! The Bevy plugin for Bones3 for setting up all systems and components
//! required to use Bones3 in a Bevy app.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::prelude::*;

/// The root plugin for implementing all Bones Cubed logic components and
/// systems.
///
/// This plugin includes components for creating VoxelWorld components, which
/// are infinite grids that store a single type of data. Normally, all static
/// block data will be stored in this data type and attached to the world. When
/// initializing the Bones3Plugin, the type `T` specifies what type of block
/// data will be stored within the voxel world components. A new instance of
/// this plugin must be defined for each block data type that is defined.
///
/// This plugin also implements systems and component support for adding chunk
/// anchors to entities. This will allow for voxel worlds to automatically load
/// and unload chunk based on the location and effect radius of chunk anchors
/// within the world.
///
/// The W component here refers to the type of world generator to use in the
/// default plugin setup. If multiple world generators are required, then the
/// plugin must be manually constructed from components.
pub struct Bones3Plugin;

impl Plugin for Bones3Plugin {
    fn build(&self, app: &mut App) {
        app
            // Storage
            .register_type::<VoxelWorld>()
            .register_type::<VoxelChunk>()
            // World Gen
            .register_type::<ChunkAnchor>()
            .register_type::<PendingLoadChunkTask>()
            .add_system(setup_chunk_transforms)
            // Query
            .register_type::<ChunkEntityPointers>();
    }
}

/// This is an addon plugin for Bones3 that adds support for a specific block
/// type.
///
/// This is required for storing block data, world generation, and so on. One
/// instance of this plugin must be created for each new instance of a block
/// data type that is required.
#[derive(Default)]
pub struct Bones3BlockTypePlugin<T>(PhantomData<T>)
where
    T: BlockData;

impl<T> Plugin for Bones3BlockTypePlugin<T>
where
    T: BlockData,
{
    fn build(&self, app: &mut App) {
        app
            // Storage
            .register_type::<VoxelStorage<T>>()
            // World Gen
            .register_type::<LoadChunkTask<T>>()
            .register_type::<WorldGeneratorHandler<T>>()
            .add_system(load_chunks_async::<T>)
            .add_system(push_chunk_async_queue::<T>)
            .add_system(finish_chunk_loading::<T>);
    }
}

/// This is an addon plugin for Bones3 that adds remesh support for chunks.
///
/// This plugin requires the `meshing` feature of Bones3 to be enabled.
#[derive(Default)]
#[cfg(feature = "meshing")]
pub struct Bones3MeshingPlugin<T>(PhantomData<T>)
where
    T: BlockData + BlockShape;

impl<T> Plugin for Bones3MeshingPlugin<T>
where
    T: BlockData + BlockShape,
{
    fn build(&self, app: &mut App) {
        app.register_type::<RemeshChunk>()
            .add_system(remesh_dirty_chunks::<T>);
    }
}
