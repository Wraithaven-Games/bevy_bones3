use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use bevy::render::render_resource::WgpuFeatures;
use bevy::render::settings::WgpuSettings;
use bevy_bones3::prelude::*;
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct BlockState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
struct ChunkWeightedDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
struct Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
struct Chunk {
    world:        Entity,
    chunk_coords: IVec3,
}

fn main() {
    println!("Press Esc to toggle cursor grabbing.");
    println!("Use WASD and Space/Shift to move.");
    println!("Use E to move the chunk anchor to the camera location.");
    println!("Use R to move the chunk weighted direction to the camera location.");
    println!("Use C to unload all existing chunks.");

    App::new()
        .insert_resource(MovementSettings {
            sensitivity: 0.00015,
            speed:       60.0,
        })
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_plugin(Bones3Plugin::<BlockState>::default())
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(init)
        .add_system(spawn_chunk_markers)
        .add_system(destroy_chunk_markers)
        .add_system(move_anchor)
        .add_system(move_anchor_dir)
        .add_system(update_weighted_dir)
        .add_system(unload_all_chunks)
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
    commands.spawn((Camera3dBundle::default(), Player, FlyCam));

    // voxel world
    let world = commands
        .spawn((
            SpatialBundle::default(),
            VoxelWorld::<BlockState>::default(),
        ))
        .id();

    // anchor
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius:  1.0,
                sectors: 24,
                stacks:  24,
            })),
            material: materials.add(Color::rgb(1.0, 0.2, 0.0).into()),
            ..default()
        },
        ChunkAnchor::new(world, 10, 16),
    ));

    // anchor weighted direction
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius:  0.5,
                sectors: 24,
                stacks:  24,
            })),
            material: materials.add(Color::rgb(0.0, 0.2, 1.0).into()),
            ..default()
        },
        ChunkWeightedDir,
    ));
}

fn spawn_chunk_markers(
    mut commands: Commands,
    mut chunk_load_ev: EventReader<ChunkLoadEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in chunk_load_ev.iter() {
        let pos = event.chunk_coords.as_vec3() * 16.0 + 8.0;
        let marker = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube {
                        size: 2.0,
                    })),
                    material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0).into()),
                    transform: Transform::from_translation(pos),
                    ..default()
                },
                Wireframe,
                Chunk {
                    world:        event.world,
                    chunk_coords: event.chunk_coords,
                },
            ))
            .id();

        commands.entity(event.world).add_child(marker);
    }
}

fn destroy_chunk_markers(
    chunk_query: Query<(Entity, &Chunk)>,
    mut commands: Commands,
    mut chunk_unload_ev: EventReader<ChunkUnloadEvent>,
) {
    for event in chunk_unload_ev.iter() {
        for (chunk_entity, chunk) in chunk_query.iter() {
            if chunk.world == event.world && chunk.chunk_coords == event.chunk_coords {
                commands.entity(chunk_entity).despawn();
                break;
            }
        }
    }
}

fn move_anchor(
    inputs: Res<Input<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    mut anchor_query: Query<&mut Transform, (With<ChunkAnchor>, Without<Player>)>,
) {
    if inputs.just_pressed(KeyCode::E) {
        let mut anchor = anchor_query.single_mut();
        let player = player_query.single();
        anchor.translation = player.translation;
    }
}

fn move_anchor_dir(
    inputs: Res<Input<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    mut anchor_dir_query: Query<&mut Transform, (With<ChunkWeightedDir>, Without<Player>)>,
) {
    if inputs.just_pressed(KeyCode::R) {
        let mut anchor_dir = anchor_dir_query.single_mut();
        let player = player_query.single();
        anchor_dir.translation = player.translation;
    }
}

fn update_weighted_dir(
    anchor_dir_query: Query<&Transform, With<ChunkWeightedDir>>,
    mut anchor_query: Query<(&mut ChunkAnchor, &Transform)>,
) {
    let (mut anchor, anchor_trans) = anchor_query.single_mut();
    let anchor_dir_trans = anchor_dir_query.single();
    anchor.set_weighted_dir((anchor_dir_trans.translation - anchor_trans.translation) / 32.0);
}

fn unload_all_chunks(
    inputs: Res<Input<KeyCode>>,
    mut chunk_unload_ev: EventWriter<ChunkUnloadEvent>,
    mut world_query: Query<(Entity, &mut VoxelWorld<BlockState>)>,
) {
    if inputs.just_pressed(KeyCode::C) {
        let (world_entity, mut world) = world_query.single_mut();
        world
            .unload_all_chunks()
            .call_event(&mut chunk_unload_ev, world_entity);
    }
}
