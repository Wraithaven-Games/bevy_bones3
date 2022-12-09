//! The Bevy plugin for Bones3 for setting up all systems and components
//! required to use Bones3 in a Bevy app.

use std::marker::PhantomData;

use bevy::prelude::{App, Plugin};

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
#[derive(Debug, Default)]
pub struct Bones3Plugin<T: BlockData, W: WorldGenerator<T>> {
    /// Phantom data for T.
    _phantom_t: PhantomData<T>,

    /// Phantom data for W.
    _phantom_w: PhantomData<W>,
}

impl<T: BlockData, W: WorldGenerator<T>> Plugin for Bones3Plugin<T, W> {
    fn build(&self, app: &mut App) {
        app.register_type::<VoxelWorld<T>>()
            .register_type::<ChunkAnchor>()
            .add_event::<ChunkLoadEvent>()
            .add_event::<ChunkUnloadEvent>()
            .add_system(load_chunks_async::<T, W>)
            .add_system(finish_chunk_loading::<T>);
    }
}
