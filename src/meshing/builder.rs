//! This module contains the core algorithm for generating a mesh from a voxel
//! storage chunk.

use bevy::prelude::*;

use super::{BlockOcclusion, BlockShape, TempMesh};
use crate::prelude::Region;
use crate::storage::BlockData;

/// Builds a temp mesh for a virtual 16x16x16 chunk with support for reading
/// block data from neighboring virtual chunks.
///
/// This method will iterator over all values within the 16x16x16 local
/// coordinates and read the corresponding block values from the `get_block`
/// parameter function provided. For neighboring chunks, values one block
/// outside of the standard local block coordinates in each of the six cubic
/// directions are also read using the `get_block` parameter function with
/// values that would lie outside of a standard chunk block coordinate.
pub fn build_chunk_mesh<T, G>(get_block: G) -> TempMesh
where
    T: BlockData + BlockShape,
    G: Fn(IVec3) -> T,
{
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

    mesh
}
