#![allow(dead_code)]

use std::ops::Mul;

use bevy::prelude::*;
use bevy_bones3::prelude::*;
use bones3_remesh::ecs::components::RemeshChunk;
use bones3_remesh::ecs::resources::ChunkMaterialList;
use bones3_remesh::mesh::block_model::{BlockOcclusion, BlockShape};
use bones3_remesh::vertex_data::{CubeModelBuilder, ShapeBuilder};
use bones3_remesh::Bones3RemeshPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Bones3CorePlugin::<BlockState>::default(),
            Bones3RemeshPlugin::<BlockState>::default(),
        ))
        .add_systems(Startup, init)
        .run();
}

#[derive(Debug, Default, Reflect, Clone, Copy)]
enum BlockState {
    #[default]
    Empty,
    HalfSlab(u16),
    Solid(u16),
}

impl BlockShape for BlockState {
    fn write_shape(&self, shape_builder: &mut ShapeBuilder) {
        match self {
            BlockState::Empty => {},
            BlockState::HalfSlab(material) => {
                shape_builder.add_shape(
                    CubeModelBuilder::new()
                        .set_size(Vec3::new(1.0, 0.5, 1.0))
                        .set_occlusion(shape_builder.get_occlusion()),
                    *material,
                );
            },
            BlockState::Solid(material) => {
                shape_builder.add_shape(
                    CubeModelBuilder::new().set_occlusion(shape_builder.get_occlusion()),
                    *material,
                );
            },
        }
    }

    // This function checks whenever two neighboring faces are blocking each other.
    // This has the purpose of culling non-visible faces, which is essential to
    // performance
    fn check_occlude(&self, face: BlockOcclusion, _other: Self) -> bool {
        match self {
            BlockState::Empty => false, // if this tile is empty, it will never block a face
            BlockState::Solid(_) => true, // solid blocks always will allays block neighboring
            // faces
            BlockState::HalfSlab(_) => BlockOcclusion::NEG_Y.contains(face), /* A halfslab only
                                                                              * blocks faces
                                                                              * below it. */
        }
    }
}

fn init(
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
    let stone_index = chunk_materials.add_material(stone_handle, None);
    let grass_handle = materials.add(Color::DARK_GREEN.into());
    let grass_index = chunk_materials.add_material(grass_handle, None);

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
            } else if vert < (shape + 0.25).floor() {
                storage.set_block(pos, BlockState::HalfSlab(material_index));
            } else {
                storage.set_block(pos, BlockState::Empty);
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
