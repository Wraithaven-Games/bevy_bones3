//! This module contains systems that will automatically trigger chunks marked
//! as dirty to be remeshed and keeping everything up to date.

use bevy::prelude::*;
use bones3_core::prelude::Region;
use bones3_core::query::VoxelQuery;
use bones3_core::storage::{BlockData, VoxelChunk, VoxelStorage};
use bones3_core::util::anchor::ChunkAnchorRecipient;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

use super::components::{ChunkMesh, RemeshChunk};
use super::resources::ChunkMaterialList;
use crate::mesh::block_model::BlockShape;
use crate::mesh::builder;
use crate::RemeshAnchor;

// pub(crate) fn push_chunk_async_queue<T>(
//     active_tasks: Query<(Entity, &RemeshChunkTask<T>)>,
//     chunks: Query<(&VoxelStorage)>,
// )

/// This system remeshes dirty voxel chunks. For all chunks with the RemeshChunk
/// component, each frame, the chunk with the highest priority value
/// will be selected for mesh generation.
pub fn remesh_dirty_chunks<T>(
    dirty_chunks: Query<
        (&ChunkAnchorRecipient<RemeshAnchor>, &VoxelChunk, Entity),
        (With<RemeshChunk>, With<VoxelStorage<T>>),
    >,
    chunk_data: VoxelQuery<&VoxelStorage<T>>,
    chunk_meshes: Query<(Entity, &Parent), With<ChunkMesh>>,
    materials: Res<ChunkMaterialList>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) where
    T: BlockData + BlockShape,
{
    let max_chunks = 2;

    for (chunk_coords, chunk_id, world_id) in get_max_chunks(&dirty_chunks, max_chunks) {
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
}

/// Gets the highest priority chunks to remesh.
fn get_max_chunks<T>(
    chunks: &Query<
        (&ChunkAnchorRecipient<RemeshAnchor>, &VoxelChunk, Entity),
        (With<RemeshChunk>, With<VoxelStorage<T>>),
    >,
    max_chunks: usize,
) -> impl Iterator<Item = (IVec3, Entity, Entity)>
where
    T: BlockData + BlockShape,
{
    let mut queue = PriorityQueue::new();

    for (anchor_recipient, chunk_meta, chunk_id) in chunks.iter() {
        let priority = match anchor_recipient.priority {
            Some(p) => p,
            None => f32::NEG_INFINITY,
        };

        queue.push(
            (chunk_meta.chunk_coords(), chunk_id, chunk_meta.world_id()),
            OrderedFloat::from(priority),
        );
    }

    queue.into_sorted_iter().take(max_chunks).map(|(e, _)| e)
}
