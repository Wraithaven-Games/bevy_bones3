//! Contains the core implementation for chunk mesh generation.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{builder, BlockShape};
use crate::prelude::{Region, VoxelQuery};
use crate::storage::{BlockData, VoxelChunk, VoxelStorage};

/// A temporary marker component that indicates that the target chunk needs to
/// be remeshed.
#[derive(Debug, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct RemeshChunk;

/// This system remeshes dirty voxel chunks. For all chunks with the RemeshChunk
/// component, each frame, the chunk with the highest priority value
/// will be selected for mesh generation.
///
/// This system will also create mesh and material handles on any chunk that do
/// currently have them yet.
pub fn remesh_dirty_chunks<T>(
    camera: Query<&Transform, With<Camera3d>>,
    shapes: VoxelQuery<&VoxelStorage<T>>,
    mut dirty_chunks: VoxelQuery<
        (
            Entity,
            &VoxelChunk,
            Option<(&Handle<Mesh>, &Handle<StandardMaterial>)>,
            &mut Visibility,
        ),
        With<RemeshChunk>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) where
    T: BlockData + BlockShape,
{
    #[cfg(feature = "trace")]
    let profiler_guard = info_span!("remesh_chunk", target = "find_next_chunk").entered();

    // TODO: Improve priority calculation compatibility.
    let camera_pos: IVec3 = camera.single().translation.as_ivec3() >> 4;
    let next_chunk = dirty_chunks.iter_mut().max_by_key(|(_, meta, ..)| {
        let distance = (meta.chunk_coords() - camera_pos).as_vec3().length();
        OrderedFloat(-distance)
    });

    #[cfg(feature = "trace")]
    profiler_guard.exit();

    let Some((chunk_id, chunk_meta, handles, mut visibility)) = next_chunk else {
        return;
    };

    let world_id = chunk_meta.world_id();
    let chunk_coords = chunk_meta.chunk_coords();
    let data_region = Region::from_points(IVec3::NEG_ONE, IVec3::ONE);

    let profiler_guard = info_span!("remesh_chunk", target = "collect_block_data").entered();
    let data = data_region
        .iter()
        .map(|offset| shapes.get_chunk(world_id, chunk_coords + offset).ok())
        .collect::<Vec<Option<&VoxelStorage<T>>>>();
    profiler_guard.exit();

    let get_block = |block_pos: IVec3| {
        let chunk_index = data_region.point_to_index(block_pos >> 4).unwrap();
        match &data[chunk_index] {
            Some(chunk) => chunk.get_block(block_pos),
            None => T::default(),
        }
    };

    #[cfg(feature = "trace")]
    let profiler_guard = info_span!("remesh_chunk", target = "update_mesh").entered();

    let mesh = builder::build_chunk_mesh(get_block);
    visibility.is_visible = !mesh.is_empty();

    match handles {
        Some((mesh_handle, _material_handle)) => {
            let bevy_mesh = meshes.get_mut(mesh_handle).unwrap();
            mesh.write_to_mesh(bevy_mesh).unwrap();

            commands.entity(chunk_id).remove::<RemeshChunk>();
        },

        None => {
            let mesh_handle = meshes.add(mesh.into_mesh());
            let material_handle = materials.add(StandardMaterial::default());

            commands
                .entity(chunk_id)
                .remove::<RemeshChunk>()
                .insert((mesh_handle, material_handle));
        },
    };

    #[cfg(feature = "trace")]
    profiler_guard.exit();
}
