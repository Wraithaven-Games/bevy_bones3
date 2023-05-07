#![allow(dead_code)]

use bevy::prelude::*;
use bevy_bones3::prelude::*;
use bevy_flycam::PlayerPlugin;
use bones3_worldgen::Bones3WorldGenPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Bones3CorePlugin::<BlockState>::default())
        .add_plugin(Bones3RemeshPlugin::<BlockState>::default())
        .add_plugin(Bones3WorldGenPlugin::<BlockState>::default())
        .add_plugin(PlayerPlugin)
        .add_startup_system(init.at_end())
        .run();
}

#[derive(Debug, Default, Clone, Copy)]
enum BlockState {
    #[default]
    Empty,
    Solid(u16),
}

impl BlockShape for BlockState {
    fn write_shape(&self, shape_builder: &mut ShapeBuilder) {
        match self {
            BlockState::Empty => {},
            BlockState::Solid(material) => {
                shape_builder.add_shape(
                    CubeModelBuilder::new().set_occlusion(shape_builder.get_occlusion()),
                    *material,
                );
            },
        }
    }

    fn get_occludes(&self) -> BlockOcclusion {
        match self {
            BlockState::Empty => BlockOcclusion::empty(),
            BlockState::Solid(_) => BlockOcclusion::all(),
        }
    }
}

fn init(
    camera: Query<Entity, With<Camera3d>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_materials: ResMut<ChunkMaterialList>,
    mut commands: VoxelCommands,
) {
    let stone_handle = materials.add(Color::WHITE.into());
    chunk_materials.add_material(stone_handle);

    let world_id = commands.spawn_world(SpatialBundle::default()).id();
    let commands = commands.commands();

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.25, 0.0)),
        directional_light: DirectionalLight {
            illuminance: 50000.0,
            ..default()
        },
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color:      Color::WHITE,
        brightness: 2.5,
    });

    commands
        .entity(camera.single())
        .insert(ChunkAnchor::new(world_id, 10, 12, 2));
}
