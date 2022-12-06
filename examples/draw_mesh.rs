use bevy::prelude::*;
use bevy_bones3::prelude::*;
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};

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

fn main() {
    println!("Press Esc to toggle cursor grabbing.");
    println!("Use WASD and Space/Shift to move.");

    App::new()
        .insert_resource(MovementSettings {
            sensitivity: 0.00015,
            speed:       12.0,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Bones3Plugin::<10, BlockState>::default())
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(init)
        .run();
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.25, 0.0)),
        ..default()
    });

    // player
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(28.0, 20.0, 32.0)
                .looking_at(Vec3::new(8.0, 8.0, 8.0), Vec3::Y),
            ..default()
        },
        FlyCam,
    ));

    // voxel world
    let mut voxel_world = VoxelWorld::<BlockState>::default();
    voxel_world.init_chunk(IVec3::ZERO).into_result().unwrap();

    for block_pos in Region::CHUNK.iter() {
        if block_pos.as_vec3().distance(Vec3::new(8.0, 8.0, 8.0)) < 8.0 {
            voxel_world.set_block(block_pos, BlockState::Solid).unwrap();
        }
    }

    let mesh = voxel_world.generate_mesh_for_chunk(IVec3::ZERO).into_mesh();
    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.0, 0.4, 0.1).into()),
        ..default()
    });
}
