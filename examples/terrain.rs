use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_bones3::prelude::*;
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum BlockState {
    #[default]
    Empty,
    Solid,
}

impl BlockShape for BlockState {
    fn get_generator(&self) -> Option<Box<dyn BlockModelGenerator>> {
        match self {
            BlockState::Empty => None,
            BlockState::Solid => Some(Box::new(CubeModelBuilder::new())),
        }
    }

    fn get_occludes(&self) -> BlockOcclusion {
        match self {
            BlockState::Empty => BlockOcclusion::empty(),
            BlockState::Solid => BlockOcclusion::all(),
        }
    }
}

#[derive(Clone, Copy, Reflect, Default)]
struct Terrain;

impl WorldGenerator<BlockState> for Terrain {
    fn generate_chunk(&self, chunk_coords: IVec3) -> VoxelStorage<BlockState> {
        let mut storage = VoxelStorage::<BlockState>::default();

        let perlin = Perlin::new(27);
        for local_pos in Region::CHUNK.iter() {
            let block_coords: IVec3 = (chunk_coords << 4) + local_pos;
            let pos = block_coords.xz().as_dvec2() / 64.0_f64;
            let height = perlin.get(pos.into()) * 16.0 - 10.0;

            if block_coords.y >= height as i32 {
                storage.set_block(local_pos, BlockState::Empty);
            } else {
                storage.set_block(local_pos, BlockState::Solid);
            }
        }

        storage
    }
}

fn main() {
    println!("Press Esc to toggle cursor grabbing.");
    println!("Use WASD and Space/Shift to move.");

    App::new()
        .insert_resource(MovementSettings {
            sensitivity: 0.00015,
            speed:       10.0,
        })
        .insert_resource(AmbientLight {
            color:      Color::WHITE,
            brightness: 2.5,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Bones3Plugin)
        .add_plugin(Bones3BlockTypePlugin::<BlockState>::default())
        .add_plugin(Bones3MeshingPlugin::<BlockState>::default())
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(init)
        .run();
}

fn init(mut commands: Commands) {
    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.25, 0.0)),
        directional_light: DirectionalLight {
            illuminance: 50000.0,
            ..default()
        },
        ..default()
    });

    // voxel world
    let world = commands
        .spawn_world::<BlockState, _>((
            SpatialBundle::default(),
            RemeshWorld,
            WorldGeneratorHandler::from(Terrain),
        ))
        .id();

    // player
    commands.spawn((
        Camera3dBundle::default(),
        FlyCam,
        ChunkAnchor::new(world, 10, 16),
    ));
}

// fn gen_chunk_meshes<T: BlockData + BlockShape>(
//     worlds: Query<&VoxelWorld<T>>,
//     mut commands: Commands,
//     mut chunk_load_ev: EventReader<ChunkLoadEvent>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     for event in chunk_load_ev.iter() {
//         let world = worlds.get(event.world).unwrap();

//         let pos = event.chunk_coords.as_vec3() * 16.0;
//         let region = Region::CHUNK.shift(event.chunk_coords << 4);
//         let Ok(mesh) = world.generate_mesh(region).into_mesh() else {
//             continue;
//         };

//         let mut material: StandardMaterial = Color::rgb(0.0, 0.4,
// 0.1).into();         material.perceptual_roughness = 0.95;
//         material.reflectance = 0.0;

//         let chunk = commands
//             .spawn(PbrBundle {
//                 mesh: meshes.add(mesh),
//                 material: materials.add(material),
//                 transform: Transform::from_translation(pos),
//                 ..default()
//             })
//             .id();

//         commands.entity(event.world).add_child(chunk);
//     }
// }
