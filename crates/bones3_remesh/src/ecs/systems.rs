//! This module contains systems that will automatically trigger chunks marked
//! as dirty to be remeshed and keeping everything up to date.

use bevy::prelude::*;
use bones3_core::prelude::Region;
use bones3_core::query::VoxelQuery;
use bones3_core::storage::{BlockData, VoxelChunk, VoxelStorage};
use ordered_float::OrderedFloat;

use crate::mesh::builder;
use crate::prelude::{
    BlockShape,
    ChunkMaterialList,
    ChunkMesh,
    ChunkMeshCameraAnchor,
    RemeshChunk,
};

/// This system remeshes dirty voxel chunks. For all chunks with the RemeshChunk
/// component, each frame, the chunk with the highest priority value
/// will be selected for mesh generation.
pub fn remesh_dirty_chunks<T>(
    anchors: Query<&GlobalTransform, With<ChunkMeshCameraAnchor>>,
    dirty_chunks: Query<(Entity, &VoxelChunk, &GlobalTransform), With<RemeshChunk>>,
    chunk_data: VoxelQuery<&VoxelStorage<T>>,
    chunk_meshes: Query<(Entity, &Parent), With<ChunkMesh>>,
    materials: Res<ChunkMaterialList>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) where
    T: BlockData + BlockShape,
{
    // Get highest priority chunk
    let Some((chunk_id, chunk_meta, _)) = dirty_chunks.iter().max_by_key(|(_, _, transform)| {
        let distance: f32 = anchors
            .iter()
            .map(|anchor| anchor.translation().distance(transform.translation()))
            .sum();
        OrderedFloat(-distance)
    }) else {
        return;
    };

    // Create block accessor
    let world_id = chunk_meta.world_id();
    let chunk_coords = chunk_meta.chunk_coords();
    let data_region = Region::from_points(IVec3::NEG_ONE, IVec3::ONE);
    let world_data_query = chunk_data.get_world(world_id).unwrap();

    let data = data_region
        .iter()
        .map(|offset| world_data_query.get_chunk(chunk_coords + offset))
        .collect::<Vec<Option<&VoxelStorage<T>>>>();

    let get_block = |block_pos: IVec3| {
        let chunk_index = data_region.point_to_index(block_pos >> 4).unwrap();
        match &data[chunk_index] {
            Some(chunk) => chunk.get_block(block_pos),
            None => T::default(),
        }
    };

    commands.entity(chunk_id).remove::<RemeshChunk>();

    let shape_builder = builder::build_chunk_mesh(get_block, &materials);
    builder::apply_shape_builder(
        chunk_id,
        shape_builder,
        &chunk_meshes,
        &mut meshes,
        &mut commands,
    );
}
