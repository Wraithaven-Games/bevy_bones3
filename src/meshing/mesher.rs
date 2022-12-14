//! Contains the core implementation for chunk mesh generation.

use bevy::prelude::*;

use super::{BlockOcclusion, BlockShape, TempMesh};
use crate::prelude::{Region, VoxelQuery};
use crate::storage::{BlockData, VoxelChunk, VoxelStorage};

/// A marker component that indicates that the target chunk needs to be
/// remeshed.
#[derive(Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct RemeshChunk;

/// This system remeshes dirty voxel chunks.
///
/// A dirty chunk is defined as a chunk with the RemeshChunk component, with
/// `needs_remesh` set to true.
pub fn remesh_dirty_chunks<T>(
    shapes: VoxelQuery<&VoxelStorage<T>>,
    dirty_chunks: VoxelQuery<(&Handle<Mesh>, &VoxelChunk), With<RemeshChunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
) where
    T: BlockData + BlockShape,
{
    for (mesh_handle, chunk) in dirty_chunks.iter() {
        let world_id = chunk.world_id();

        let chunk_coords = chunk.chunk_coords();
        let data_region = Region::from_points(IVec3::NEG_ONE, IVec3::ONE);

        let data = data_region
            .iter()
            .map(|offset| shapes.get_chunk(world_id, chunk_coords + offset).ok())
            .collect::<Vec<Option<&VoxelStorage<T>>>>();

        let get_block = |block_pos: IVec3| {
            let chunk_index = data_region.point_to_index(block_pos >> 4).unwrap();
            match &data[chunk_index] {
                Some(chunk) => chunk.get_block(block_pos),
                None => T::default(),
            }
        };

        let mut mesh = TempMesh::default();
        for block_pos in Region::CHUNK.iter() {
            let data = get_block(block_pos);
            let Some(mut model_gen) = data.get_generator() else {
                continue;
            };

            let check_occlusion = |occlusion: &mut BlockOcclusion, face: BlockOcclusion| {
                if get_block(block_pos + face.into_offset())
                    .get_occludes()
                    .contains(face.opposite_face())
                {
                    occlusion.insert(face)
                }
            };

            let mut occlusion = BlockOcclusion::empty();
            check_occlusion(&mut occlusion, BlockOcclusion::NEG_X);
            check_occlusion(&mut occlusion, BlockOcclusion::POS_X);
            check_occlusion(&mut occlusion, BlockOcclusion::NEG_Y);
            check_occlusion(&mut occlusion, BlockOcclusion::POS_Y);
            check_occlusion(&mut occlusion, BlockOcclusion::NEG_Z);
            check_occlusion(&mut occlusion, BlockOcclusion::POS_Z);

            model_gen.set_block_pos(block_pos);
            model_gen.set_occlusion(occlusion);
            model_gen.write_to_mesh(&mut mesh);
        }

        let bevy_mesh = meshes.get_mut(mesh_handle).unwrap();
        mesh.write_to_mesh(bevy_mesh).unwrap();
    }
}
