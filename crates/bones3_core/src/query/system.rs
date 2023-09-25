//! The Bevy system parameter value.

use bevy::ecs::query::{QueryItem, ROQueryItem, ReadOnlyWorldQuery, WorldQuery};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::VoxelQueryError;
use crate::storage::chunk_pointers::ChunkEntityPointers;
use crate::storage::{VoxelChunk, VoxelWorld};

/// A system parameter designed for quickly querying and reading and writing to
/// voxel worlds and voxel chunks.
#[derive(SystemParam)]
pub struct VoxelQuery<'w, 's, Q, F = ()>
where
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// A readonly query of chunk entity pointers.
    chunk_pointers: Query<'w, 's, (Entity, &'static ChunkEntityPointers), With<VoxelWorld>>,

    /// A standard query of voxel chunks.
    query: Query<'w, 's, (&'static VoxelChunk, Q), (With<VoxelChunk>, F)>,
}

impl<'w, 's, 'a, Q, F> VoxelQuery<'w, 's, Q, F>
where
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// Creates a readonly iterator over all chunks thaT match the given system
    /// query.
    pub fn iter(&'a self) -> impl Iterator<Item = ROQueryItem<'_, Q>> + '_ {
        self.query.iter().map(|(_, q)| q)
    }

    /// Creates a mutable iterator over all chunks thaT match the given system
    /// query.
    pub fn iter_mut(&'a mut self) -> impl Iterator<Item = QueryItem<'_, Q>> + '_ {
        self.query.iter_mut().map(|(_, q)| q)
    }

    /// Gets a readonly reference to the voxel world with the given world id.
    /// The world may or may not have any chunks in it that match the given
    /// system query.
    pub fn get_world(
        &'a self,
        world_id: Entity,
    ) -> Result<VoxelWorldQuery<'w, 's, 'a, Q, F>, VoxelQueryError> {
        self.chunk_pointers
            .get(world_id)
            .map_err(|_| VoxelQueryError::WorldNotFound(world_id))?;

        Ok(VoxelWorldQuery {
            voxel_query: self,
            world_id,
        })
    }

    /// Gets a mutable reference to the voxel world with the given world id. The
    /// world may or may not have any chunks in it that match the given system
    /// query.
    pub fn get_world_mut(
        &'a mut self,
        world_id: Entity,
    ) -> Result<VoxelWorldQueryMut<'w, 's, 'a, Q, F>, VoxelQueryError> {
        self.chunk_pointers
            .get(world_id)
            .map_err(|_| VoxelQueryError::WorldNotFound(world_id))?;

        Ok(VoxelWorldQueryMut {
            voxel_query: self,
            world_id,
        })
    }

    /// Gets a readonly iterator over all voxel worlds.
    ///
    /// Note: This also includes worlds that do not contain any valid chunks
    /// within the system query.
    pub fn world_iter(&'a self) -> impl Iterator<Item = VoxelWorldQuery<'w, 's, 'a, Q, F>> + '_ {
        self.chunk_pointers.iter().map(|(id, _)| {
            VoxelWorldQuery {
                voxel_query: self,
                world_id:    id,
            }
        })
    }
}

/// A readonly utility handler for querying chunks within a specific voxel
/// world.
pub struct VoxelWorldQuery<'w, 's, 'a, Q, F>
where
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// A reference to the voxel query that owns the chunk query itself.
    voxel_query: &'a VoxelQuery<'w, 's, Q, F>,

    /// The id of the world that is being handled.
    world_id: Entity,
}

impl<'w, 's, 'a, Q, F> VoxelWorldQuery<'w, 's, 'a, Q, F>
where
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// Creates a readonly iterator over all chunks within this world that match
    /// the query.
    ///
    /// This method is implemented by applying a filter the top of the standard
    /// query iterator. As such, calling them method for multiple worlds
    /// might be slower than calling [`VoxelQuery::iter`] directly.
    pub fn iter(&'a self) -> impl Iterator<Item = ROQueryItem<'_, Q>> + '_ {
        self.voxel_query
            .query
            .iter()
            .filter(|(c, _)| c.world_id() == self.world_id)
            .map(|(_, q)| q)
    }

    /// Gets the chunk at the given chunk coordinates within this world, if it
    /// is both loaded and matches the indicated system query. Otherwise,
    /// this method returns None.
    pub fn get_chunk(&'a self, chunk_coords: IVec3) -> Option<ROQueryItem<'_, Q>> {
        let chunk_id = self
            .voxel_query
            .chunk_pointers
            .get(self.world_id)
            .map(|(_, p)| p)
            .unwrap()
            .get_chunk_entity(chunk_coords)?;

        self.voxel_query.query.get(chunk_id).ok().map(|(_, q)| q)
    }

    /// Gets the chunk at the given block coordinates within this world, if it
    /// is both loaded and matches the indicated system query. Otherwise,
    pub fn get_chunk_at_block(&'a mut self, block_coords: IVec3) -> Option<ROQueryItem<'_, Q>> {
        self.get_chunk(block_coords >> 4)
    }

    /// Gets the id of the voxel world being handled.
    pub fn world_id(&self) -> Entity {
        self.world_id
    }
}

/// A mutable utility handler for querying chunks within a specific voxel world.
pub struct VoxelWorldQueryMut<'w, 's, 'a, Q, F>
where
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// A mutable reference to the voxel query that owns the chunk query itself.
    voxel_query: &'a mut VoxelQuery<'w, 's, Q, F>,

    /// The id of the world that is being handled.
    world_id: Entity,
}

impl<'w, 's, 'a, Q, F> VoxelWorldQueryMut<'w, 's, 'a, Q, F>
where
    Q: WorldQuery + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    /// Creates a mutable iterator over all chunks within this world that match
    /// the query.
    ///
    /// This method is implemented by applying a filter the top of the standard
    /// query iterator. As such, calling them method for multiple worlds
    /// might be slower than calling [`VoxelQuery::iter_mut`] directly.
    pub fn iter_mut(&'a mut self) -> impl Iterator<Item = QueryItem<'_, Q>> + '_ {
        self.voxel_query
            .query
            .iter_mut()
            .filter(|(c, _)| c.world_id() == self.world_id)
            .map(|(_, q)| q)
    }

    /// Gets the chunk at the given chunk coordinates within this world,
    /// mutably, if it is both loaded and matches the indicated system query.
    /// Otherwise, this method returns None.
    pub fn get_chunk_mut(&'a mut self, chunk_coords: IVec3) -> Option<QueryItem<'_, Q>> {
        let chunk_id = self
            .voxel_query
            .chunk_pointers
            .get(self.world_id)
            .map(|(_, p)| p)
            .unwrap()
            .get_chunk_entity(chunk_coords)?;

        self.voxel_query
            .query
            .get_mut(chunk_id)
            .ok()
            .map(|(_, q)| q)
    }

    /// Gets the chunk at the given block coordinates within this world,
    /// mutably, if it is both loaded and matches the indicated system query.
    /// Otherwise, this method returns None.
    pub fn get_chunk_at_block_mut(&'a mut self, block_coords: IVec3) -> Option<QueryItem<'_, Q>> {
        self.get_chunk_mut(block_coords >> 4)
    }

    /// Gets the id of the voxel world being handled.
    pub fn world_id(&self) -> Entity {
        self.world_id
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::prelude::VoxelCommands;

    #[test]
    fn iter_chunks_in_world() {
        let mut app = App::new();

        #[derive(Component)]
        struct WorldMarker;

        #[derive(Component)]
        struct ChunkMarker;

        fn init(mut commands: VoxelCommands) {
            let mut world_a = commands.spawn_world(WorldMarker);
            world_a.spawn_chunk(IVec3::ZERO, ChunkMarker).unwrap();
            world_a.spawn_chunk(IVec3::ONE, ChunkMarker).unwrap();
            world_a.spawn_chunk(IVec3::NEG_X, ()).unwrap();

            let mut world_b = commands.spawn_world(());
            world_b.spawn_chunk(IVec3::ZERO, ChunkMarker).unwrap();
            world_b.spawn_chunk(IVec3::ONE, ChunkMarker).unwrap();
            world_b.spawn_chunk(IVec3::NEG_X, ()).unwrap();
        }
        Schedule::new().add_systems(init).run(&mut app.world);

        fn update(
            world_query: Query<Entity, With<WorldMarker>>,
            chunk_query: VoxelQuery<&VoxelChunk, With<ChunkMarker>>,
        ) {
            let world_id = world_query.get_single().unwrap();
            let single_world = chunk_query.get_world(world_id).unwrap();
            let mut iter = single_world.iter();

            assert_eq!(iter.next(), Some(&VoxelChunk::new(world_id, IVec3::ZERO)));
            assert_eq!(iter.next(), Some(&VoxelChunk::new(world_id, IVec3::ONE)));
            assert_eq!(iter.next(), None);
        }
        Schedule::new().add_systems(update).run(&mut app.world);
    }
}
