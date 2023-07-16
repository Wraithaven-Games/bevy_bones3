#![allow(dead_code)]

use bevy::prelude::*;
use bevy_bones3::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Bones3CorePlugin::<BlockState>::default())
        .add_systems(Startup, init)
        .add_systems(Update, update)
        .run();
}

// When specifying a block data container, it needs to implement the Default,
// Clone, and Copy traits.
#[derive(Debug, Default, Reflect, Clone, Copy)]
struct BlockState {
    // Any further data can be stored inside and an instance of it will be stored
    // for every grid value within the infinite world. Here, the values being
    // stored are an enum value that determines the furniture at the current
    // location, if any, as well as the air density and light value. A block
    // data container can be as complex as it needs to be. Just watch out for
    // memory size. Each world chunk, stored in 16x16x16 groups, will consume
    // N x 4096 bytes, where N is the memory size of a block data object.
    pub furniture:   FurnitureValue,
    pub air_density: f32,
    pub light_value: i32,
}

#[derive(Debug, Default, Reflect, Clone, Copy)]
enum FurnitureValue {
    #[default]
    None,
    Chair,
    Table,
    Stool,
}

#[derive(Component)]
struct LightSource {
    pub world_id: Entity,
    pub pos:      IVec3,
}

// Bones Cubed provides an alternative system parameter for Commands and
// Queries, VoxelCommands and VoxelQueries respectfully, that are designed to
// aid in working with worlds and chunks much easier.
fn init(mut commands: VoxelCommands) {
    // It is recommended to create a new world in a startup system, as worlds and
    // chunks require a stage to be completed before they actually spawn. However,
    // chunk creation and block manipulation can be queued alongside the world
    // creation for easier startup handling.
    let mut world_cmd = commands.spawn_world(());
    let world_id = world_cmd.id();

    // The voxel storage component is used to store a 16x16x16 grid of data within a
    // chunk. By default it is filled with the default value for BlockState. This
    // voxel storage component can be added to a new chunk to spawn a chunk with
    // this data.
    let mut voxel_storage = VoxelStorage::<BlockState>::default();
    voxel_storage.set_block(IVec3::new(12, 6, 2), BlockState {
        furniture:   FurnitureValue::Chair,
        air_density: 1.2,
        light_value: 13,
    });

    // Spawning chunks is straightforward with VoxelCommands, allowing support for
    // spawning chunks at a given chunk coordinate location as well as the component
    // bundle to attach to it. Note that two chunks cannot exist at the same
    // location at once. This method will return an error if attempting to do so, or
    // will cause a panic if two chunks are queued to be spawned on the same frame
    // that will overlap.
    world_cmd
        .spawn_chunk(IVec3::new(1, 2, 3), voxel_storage)
        .unwrap();

    // The `commands()` function can be called to return the underlying Bevy command
    // queue.
    commands.commands().spawn(LightSource {
        world_id,
        pos: IVec3::new(1, 1, 1),
    });
}

// When using a VoxelQuery, it will automatically look for any chunks that match
// the given query. The filter `With<VoxelChunk>` is assumed to be attached by
// default, although additional filters can still be applied.
//
// VoxelQueries have usefulness over standard Queries, in which they are
// coordinate and world aware for each of the chunks. Chunks within the query
// can
fn update(
    light_sources: Query<&LightSource>,
    mut query: VoxelQuery<&mut VoxelStorage<BlockState>>,
) {
    // Randomize air density
    for mut storage in query.iter_mut() {
        let pos = IVec3::new(
            rand::random::<i32>() % 16,
            rand::random::<i32>() % 16,
            rand::random::<i32>() % 16,
        );

        let mut state = storage.get_block(pos);
        state.air_density = rand::random::<f32>();

        storage.set_block(pos, state);
    }

    // Update light sources
    for light_source in light_sources.iter() {
        let mut world = query.get_world_mut(light_source.world_id).unwrap();
        let chunk = world.get_chunk_at_block_mut(light_source.pos);

        if let Some(mut storage) = chunk {
            let mut state = storage.get_block(light_source.pos);
            state.light_value += 1;
            storage.set_block(light_source.pos, state);
        }
    }
}
