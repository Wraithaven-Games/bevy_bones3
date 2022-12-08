//! This module handles the automatic loading and unloading of chunks within a
//! voxel world based off a given chunk anchor's position and effect radius.

mod cache;
mod component;
mod systems;

pub use component::ChunkAnchor;
pub use systems::*;

#[cfg(test)]
mod test {
    use bevy::prelude::*;
    use bevy_test_utils::TestApp;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::prelude::ChunkLoadEvent;
    use crate::storage::VoxelWorld;

    #[test]
    fn load_chunks() {
        let mut app = App::new();
        app.add_event::<ChunkLoadEvent>();
        app.add_plugins(MinimalPlugins);

        let world = app
            .world
            .spawn((TransformBundle::default(), VoxelWorld::<u8>::default()))
            .id();

        app.world.spawn((
            Transform {
                translation: Vec3::new(17.0, -2.0, 3.0),
                ..default()
            },
            ChunkAnchor::new(world, 2, 3),
        ));

        app.run_system_once(systems::load_chunks::<10, u8>);

        let load_chunk_ev = app.world.resource::<Events<ChunkLoadEvent>>();
        let mut load_chunk_reader = load_chunk_ev.get_reader();
        let mut iter = load_chunk_reader.iter(load_chunk_ev);

        let chunk_load = iter.next().unwrap();
        assert_eq!(chunk_load.world, world);
        assert_eq!(chunk_load.chunk_coords, IVec3::new(1, -1, 0));

        assert_eq!(iter.next(), None);
    }
}
