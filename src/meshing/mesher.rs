//! Contains the core implementation for chunk mesh generation.

use crate::prelude::*;

/// A trait that is applied to a voxel world to all for chunks to be remeshed.
pub trait RemeshChunk {
    /// Generates a new mesh for the blocks located within the given region.
    fn generate_mesh(&self, region: Region) -> TempMesh;
}

impl<T: BlockData + BlockShape> RemeshChunk for VoxelWorld<T> {
    fn generate_mesh(&self, region: Region) -> TempMesh {
        let mut mesh = TempMesh::default();

        let blocks = self.get_slice(Region::from_points(region.min() - 1, region.max() + 1));

        for block_pos in region.iter() {
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

            let local_pos = block_pos - region.min();
            model_gen.set_block_pos(local_pos);
            model_gen.set_occlusion(occlusion);
            model_gen.write_to_mesh(&mut mesh);
        }

        mesh
    }
}
