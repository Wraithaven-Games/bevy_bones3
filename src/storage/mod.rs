//! This module contains the basic storage functionality used for reading and
//! writing block data.

use bevy::prelude::*;

mod block;
mod block_list;
mod events;
mod systems;
mod voxel_chunk;
mod voxel_world;
mod voxeldata_param;

pub use block::*;
pub use block_list::*;
pub use events::*;
pub use voxel_chunk::*;
pub use voxel_world::*;
pub use voxeldata_param::*;

/// The Bones3 sub-plugins for managing storage systems.
pub(crate) struct StoragePlugin;
impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetBlockEvent>()
            .add_systems(PostUpdate, systems::set_block);
    }
}
