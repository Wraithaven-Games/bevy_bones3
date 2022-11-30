//! Contains the implementation for VoxelWorld data storage and manipulation.
//!
//! This module only refers to the current in-memory state of the world.
//! Unloaded sections of the world must be loaded before they can be properly
//! manipulated.


mod chunk;
mod sector;
mod voxel;
mod world;

pub use voxel::{BlockData, ChunkLoad, VoxelStorage};
pub use world::VoxelWorld;


#[cfg(test)]
mod test {
    use super::*;
    use bevy::prelude::*;
    use pretty_assertions::assert_eq;


    #[test]
    fn read_write_world() {
        let mut world = VoxelWorld::<u8>::default();
        let pos = IVec3::new(15, 128, -3);

        world.init_chunk(pos >> 4).unwrap();
        assert_eq!(world.get_block(pos).unwrap(), 0);

        world.set_block(pos, 7).unwrap();
        assert_eq!(world.get_block(pos).unwrap(), 7);
    }
}
