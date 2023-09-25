use std::ops::Mul;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
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
        .add_systems(
            Update,
            update_wave.run_if(on_timer(Duration::from_secs_f32(0.25))),
        )
        .run();
}

#[derive(Debug, Default, Reflect, Clone, Copy)]
enum BlockState {
    #[default]
    Empty,
    Solid,
}

impl BlockShape for BlockState {
    fn write_shape(&self, shape_builder: &mut ShapeBuilder) {
        match self {
            BlockState::Empty => {},
            BlockState::Solid => {
                shape_builder.add_shape(
                    CubeModelBuilder::new().set_occlusion(shape_builder.get_occlusion()),
                    0,
                );
            },
        }
    }

    fn check_occlude(&self, _: BlockOcclusion, _other: Self) -> bool {
        match self {
            BlockState::Empty => false,
            BlockState::Solid => true,
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
    chunk_materials.add_material(stone_handle, None);

    let mut world = commands.spawn_world(SpatialBundle::default());

    let chunk_radius = IVec3::new(3, 0, 3);
    for chunk_coords in Region::from_points(-chunk_radius, chunk_radius).iter() {
        world
            .spawn_chunk(
                chunk_coords,
                (
                    SpatialBundle {
                        transform: Transform::from_translation(chunk_coords.as_vec3() * 16.0),
                        ..default()
                    },
                    VoxelStorage::<BlockState>::default(),
                ),
            )
            .unwrap();
    }
}

fn update_wave(
    time: Res<Time>,
    mut chunks: Query<(&mut VoxelStorage<BlockState>, &VoxelChunk, Entity)>,
    mut commands: Commands,
) {
    for (_, _, chunk_id) in chunks.iter() {
        commands.entity(chunk_id).insert(RemeshChunk);
    }

    let secs = time.elapsed_seconds();

    chunks
        .par_iter_mut()
        .for_each_mut(|(mut storage, chunk_meta, _)| {
            let chunk_coords = chunk_meta.chunk_coords();
            for pos in Region::CHUNK.shift(chunk_coords * 16).iter() {
                let distance = pos
                    .as_vec3()
                    .mul(Vec3::new(1.0, 0.0, 1.0))
                    .distance(Vec3::new(7.5, 0.0, 7.5));

                let shape = ((distance * distance) / 128.0 - secs).sin() * 4.0 + 7.0;
                let vert = pos.y as f32;

                if vert < shape.floor() {
                    storage.set_block(pos, BlockState::Solid);
                } else {
                    storage.set_block(pos, BlockState::Empty);
                }
            }
        });
}
