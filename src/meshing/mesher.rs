//! Contains the core implementation for chunk mesh generation.

use bevy::prelude::IVec3;

use crate::prelude::*;

/// A trait that is applied to a voxel world to all for chunks to be remeshed.
pub trait RemeshChunk {
    /// Generates a new mesh for the chunk at the given chunk coordinates.
    fn generate_mesh_for_chunk(&self, chunk_coords: IVec3) -> TempMesh;
}

impl<T: BlockData + BlockShape> RemeshChunk for VoxelWorld<T> {
    fn generate_mesh_for_chunk(&self, chunk_coords: IVec3) -> TempMesh {
        let mut mesh = TempMesh::default();

        let block_coords = chunk_coords << 4;
        let blocks = self.get_block_region(Region::from_points(
            block_coords + IVec3::ONE * 17,
            block_coords - IVec3::ONE,
        ));

        for local_pos in Region::CHUNK.iter() {
            let block_pos = local_pos + block_coords;
            let data = blocks.get_block(block_pos).unwrap();
            let Some(mut model_gen) = data.get_generator() else {
                continue;
            };

            let check_occlusion = |occlusion: &mut BlockOcclusion, face: BlockOcclusion| {
                if blocks
                    .get_block(block_pos + face.into_offset())
                    .unwrap()
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

            model_gen.set_block_pos(local_pos);
            model_gen.set_occlusion(occlusion);
            model_gen.write_to_mesh(&mut mesh);
        }

        mesh
    }
}
