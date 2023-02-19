#![allow(dead_code)]

use std::ops::Mul;

use bevy::prelude::*;
use bevy_bones3::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Bones3CorePlugin::<BlockState>::default())
        .add_plugin(Bones3RemeshPlugin::<BlockState>::default())
        .add_startup_system(init)
        .run();
}

#[derive(Debug, Default, Clone, Copy)]
pub enum BlockState {
    #[default]
    Empty,
    Solid(u16),
    Liquid(u16),
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
            BlockState::Liquid(material) => {
                shape_builder.add_shape(
                    CubeModelBuilder::new().set_occlusion(shape_builder.get_occlusion()),
                    *material,
                );
            },
        }
    }

    // transparency is a bit harder to achive, but here is how:
    fn check_occlude(&self, _: BlockOcclusion, other: Self) -> bool {
        match self {
            BlockState::Empty => false,
            BlockState::Solid(_) => {
                match other {
                    BlockState::Solid(_) => true,
                    BlockState::Empty => false,
                    BlockState::Liquid(_) => true,
                }
            },
            BlockState::Liquid(_) => {
                match other {
                    BlockState::Solid(_) => false,
                    BlockState::Empty => false,
                    BlockState::Liquid(_) => true,
                }
            },
        }
    }
}

pub fn init(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_materials: ResMut<ChunkMaterialList>,
    mut commands: VoxelCommands,
) {
    commands
        .commands()
        .spawn(DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.25, 0.0)),
            directional_light: DirectionalLight {
                illuminance: 50000.0,
                ..default()
            },
            ..default()
        })
        .commands()
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(75.0, 45.0, 75.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .commands()
        .insert_resource(AmbientLight {
            color:      Color::WHITE,
            brightness: 2.5,
        });

    let stone_handle = materials.add(Color::GRAY.into());
    let stone_index = chunk_materials.add_material(stone_handle);
    let grass_handle = materials.add(Color::DARK_GREEN.into());
    let grass_index = chunk_materials.add_material(grass_handle);
    let water_handle = materials.add(
        Color::Rgba {
            red:   0.0,
            green: 0.0,
            blue:  0.8,
            alpha: 0.8,
        }
        .into(),
    );
    let water_index = chunk_materials.add_material(water_handle);

    let mut world = commands.spawn_world(SpatialBundle::default());

    let chunk_radius = IVec3::new(3, 0, 3);
    for chunk_coords in Region::from_points(-chunk_radius, chunk_radius).iter() {
        let mut storage = VoxelStorage::<BlockState>::default();
        for pos in Region::CHUNK.shift(chunk_coords * 16).iter() {
            let distance = pos
                .as_vec3()
                .mul(Vec3::new(1.0, 0.0, 1.0))
                .distance(Vec3::new(7.5, 0.0, 7.5));
            let shape = ((distance * distance) / 128.0).sin() * 4.0 + 7.0;
            let vert = pos.y as f32;

            let material_index = if shape - vert <= 2.0 { grass_index } else { stone_index };

            if vert < shape.floor() {
                storage.set_block(pos, BlockState::Solid(material_index));
            } else if vert + 0.5 > shape.floor() && vert < 10.0 {
                storage.set_block(pos, BlockState::Liquid(water_index));
            } else {
                storage.set_block(pos, BlockState::Empty)
            }
        }

        world
            .spawn_chunk(
                chunk_coords,
                (
                    SpatialBundle {
                        transform: Transform::from_translation(chunk_coords.as_vec3() * 16.0),
                        ..default()
                    },
                    storage,
                    RemeshChunk,
                ),
            )
            .unwrap();
    }
}
