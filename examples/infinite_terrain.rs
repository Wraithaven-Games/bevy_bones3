use bevy::prelude::*;
use bevy_bones3::prelude::*;
use bones3_core::util::anchor::ChunkAnchor;
use bones3_remesh::ecs::resources::ChunkMaterialList;
use bones3_remesh::mesh::block_model::{BlockOcclusion, BlockShape};
use bones3_remesh::vertex_data::{CubeModelBuilder, ShapeBuilder};
use bones3_remesh::{Bones3RemeshPlugin, RemeshAnchor};
use bones3_worldgen::ecs::components::{WorldGenerator, WorldGeneratorHandler};
use bones3_worldgen::{Bones3WorldGenPlugin, WorldGenAnchor};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Bones3CorePlugin::<BlockState>::default(),
            Bones3RemeshPlugin::<BlockState>::default(),
            Bones3WorldGenPlugin::<BlockState>::default(),
        ))
        .add_systems(Startup, init)
        .run();
}

#[derive(Debug, Default, Reflect, Clone, Copy)]
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

    fn check_occlude(&self, _: BlockOcclusion, _: Self) -> bool {
        match self {
            BlockState::Empty => false,
            BlockState::Solid(_) => true,
        }
    }
}

struct GrassyHillsWorld {
    material_index: u16,
}

impl WorldGenerator<BlockState> for GrassyHillsWorld {
    fn generate_chunk(&self, chunk_coords: IVec3) -> VoxelStorage<BlockState> {
        let mut block_storage = VoxelStorage::default();

        for block_pos in Region::CHUNK.shift(chunk_coords * 16).iter() {
            let pos = block_pos.as_vec3();
            let block_state = if pos.y <= f32::sin(pos.x / 64.0) * f32::sin(pos.z / 64.0) * 16.0 {
                BlockState::Solid(self.material_index)
            } else {
                BlockState::Empty
            };

            block_storage.set_block(block_pos, block_state);
        }

        block_storage
    }
}

fn init(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_materials: ResMut<ChunkMaterialList>,
    mut commands: VoxelCommands,
) {
    let stone_handle = materials.add(Color::WHITE.into());
    let stone_index = chunk_materials.add_material(stone_handle, None);

    let world_id = commands
        .spawn_world((
            SpatialBundle::default(),
            WorldGeneratorHandler::from(GrassyHillsWorld {
                material_index: stone_index,
            }),
        ))
        .id();

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

    commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 32.0, 0.0),
                ..default()
            },
            ChunkAnchor::<WorldGenAnchor>::new(
                world_id,
                UVec3::new(10, 10, 10),
            ),
            ChunkAnchor::<RemeshAnchor>::new(
                world_id,
                UVec3::new(10, 10, 10),
            )
        ));
}
