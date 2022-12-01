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
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::prelude::ChunkLoadEvent;
    use crate::storage::VoxelWorld;
    use crate::Bones3Plugin;

    #[test]
    fn load_chunks() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugin(Bones3Plugin::<4, u8>::default());

        let world = app.world.spawn((TransformBundle::default(), VoxelWorld::<u8>::default())).id();
        app.world.spawn((
            Transform {
                translation: Vec3::new(17.0, -2.0, 3.0),
                ..default()
            },
            ChunkAnchor::new(world, 2, 3),
        ));

        app.update();

        let load_chunk_ev = app.world.resource::<Events<ChunkLoadEvent>>();
        let mut load_chunk_reader = load_chunk_ev.get_reader();
        let mut iter = load_chunk_reader.iter(load_chunk_ev);

        let chunk_load = iter.next().unwrap();
        assert_eq!(chunk_load.world, world);
        assert_eq!(chunk_load.chunk_coords, IVec3::new(1, -1, 0));

        assert_eq!(iter.next(), None);
    }
}
