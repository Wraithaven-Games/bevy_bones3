//! Contains the implementation for VoxelWorld data storage and manipulation.
//!
//! This module only refers to the current in-memory state of the world.
//! Unloaded sections of the world must be loaded before they can be properly
//! manipulated.

mod events;
mod voxel;
mod world;
mod world_slice;

pub use events::*;
pub use voxel::{BlockData, ChunkStorage, VoxelStorage, VoxelStorageRegion};
pub use world::VoxelWorld;
pub use world_slice::VoxelWorldSlice;

#[cfg(test)]
mod test {
    use bevy::prelude::*;
    use bevy_test_utils::TestApp;
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;
    use crate::prelude::{ChunkLoadEvent, ChunkUnloadEvent};
    use crate::Bones3Plugin;

    #[derive(Resource)]
    struct ChunkCoords(IVec3);

    #[test]
    fn read_write_world() {
        let mut world = VoxelWorld::<u8>::default();
        let pos = IVec3::new(15, 128, -3);

        world.init_chunk(pos >> 4).into_result().unwrap();
        assert_eq!(world.get_block(pos), 0);

        world.set_block(pos, 7).unwrap();
        assert_eq!(world.get_block(pos), 7);
    }

    #[test]
    fn load_unload_chunks() {
        // Init chunk system
        fn init_chunk<T: BlockData>(
            coords: Res<ChunkCoords>,
            mut world_query: Query<(Entity, &mut VoxelWorld<T>)>,
            mut chunk_load_ev: EventWriter<ChunkLoadEvent>,
        ) {
            let (entity, mut world) = world_query.single_mut();
            world
                .init_chunk(coords.0)
                .call_event(&mut chunk_load_ev, entity)
                .unwrap();
        }

        // Unload chunk system
        fn unload_chunk<T: BlockData>(
            coords: Res<ChunkCoords>,
            mut world_query: Query<(Entity, &mut VoxelWorld<T>)>,
            mut chunk_unload_ev: EventWriter<ChunkUnloadEvent>,
        ) {
            let (entity, mut world) = world_query.single_mut();
            world
                .unload_chunk(coords.0)
                .call_event(&mut chunk_unload_ev, entity)
                .unwrap();
        }

        // Initialize our app
        let mut app = App::new();
        app.insert_resource(ChunkCoords(IVec3::new(1, 2, -3)));
        app.add_plugin(Bones3Plugin::<4, u8>::default());

        // Initialize our test world
        app.world.spawn(VoxelWorld::<u8>::default());

        // Initialize our chunk and make sure it triggered the event.
        app.run_system_once(init_chunk::<u8>);
        let mut events = app.collect_events::<ChunkLoadEvent>();
        assert_ne!(events.next(), None); // First option should not be None
        assert_eq!(events.next(), None); // Second option should be None

        // Unload our chunk and make sure it triggered the event.
        app.run_system_once(unload_chunk::<u8>);
        let mut events = app.collect_events::<ChunkUnloadEvent>();
        assert_ne!(events.next(), None); // First option should not be None
        assert_eq!(events.next(), None); // Second option should be None
    }
}
