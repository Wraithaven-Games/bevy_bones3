//! Contains lookup tables and block model builders for generating cubic shapes
//! for block models.

use bevy::prelude::{IVec3, Vec2, Vec3};

use crate::meshing::block_model::{BlockModelGenerator, BlockOcclusion, TempMesh};

/// Contains the vertex data for generating a cube.
///
/// The vertex data is laid out as an array of vertices, where each vertex is
/// stored as a tuple containing the vertex position, normal, and uv, in that
/// order. Each of these vertex attributes is stored as a small array of floats,
/// based on the size of the data stored in that attribute for a single vertex.
///
/// The vertex are laid out in six groups, where each group contains 4 vertices,
/// pertaining to a single face of the cube. The face groups are stored in the
/// order: -X, +X, -Y, +Y, -Z, +Z. In addition, each face group uses a quad
/// index layout, as determined by [`QUAD_INDICES`].
#[rustfmt::skip]
const CUBE_VERTICES: [(Vec3, Vec3, Vec2); 24] = [
    // -X
    (Vec3::new(0.0, 0.0, 0.0), Vec3::new(-1., 0.0, 0.0), Vec2::new(0.0, 0.0)),
    (Vec3::new(0.0, 0.0, 1.0), Vec3::new(-1., 0.0, 0.0), Vec2::new(0.0, 1.0)),
    (Vec3::new(0.0, 1.0, 1.0), Vec3::new(-1., 0.0, 0.0), Vec2::new(1.0, 1.0)),
    (Vec3::new(0.0, 1.0, 0.0), Vec3::new(-1., 0.0, 0.0), Vec2::new(1.0, 0.0)),
    // +X
    (Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec2::new(0.0, 0.0)),
    (Vec3::new(1.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec2::new(0.0, 1.0)),
    (Vec3::new(1.0, 1.0, 1.0), Vec3::new(1.0, 0.0, 0.0), Vec2::new(1.0, 1.0)),
    (Vec3::new(1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0), Vec2::new(1.0, 0.0)),
    // -Y
    (Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, -1., 0.0), Vec2::new(0.0, 0.0)),
    (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, -1., 0.0), Vec2::new(0.0, 1.0)),
    (Vec3::new(1.0, 0.0, 1.0), Vec3::new(0.0, -1., 0.0), Vec2::new(1.0, 1.0)),
    (Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, -1., 0.0), Vec2::new(1.0, 0.0)),
    // +Y
    (Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec2::new(0.0, 0.0)),
    (Vec3::new(0.0, 1.0, 1.0), Vec3::new(0.0, 1.0, 0.0), Vec2::new(0.0, 1.0)),
    (Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 1.0, 0.0), Vec2::new(1.0, 1.0)),
    (Vec3::new(1.0, 1.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec2::new(1.0, 0.0)),
    // -Z
    (Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.), Vec2::new(0.0, 0.0)),
    (Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, -1.), Vec2::new(0.0, 1.0)),
    (Vec3::new(1.0, 1.0, 0.0), Vec3::new(0.0, 0.0, -1.), Vec2::new(1.0, 1.0)),
    (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.), Vec2::new(1.0, 0.0)),
    // +Z
    (Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new(0.0, 0.0)),
    (Vec3::new(1.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new(0.0, 1.0)),
    (Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new(1.0, 1.0)),
    (Vec3::new(0.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 1.0), Vec2::new(1.0, 0.0)),
];

/// The relative indices that are used to indicate how the vertices of a quad
/// are applied to write to a mesh with the TriangleList topology.
const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

/// A block model builder for a cube.
///
/// This builder is designed to make it easier to write a custom cube model to a
/// temp mesh.
pub struct CubeModelBuilder {
    /// The location of the cube model within the chunk.
    block_pos: IVec3,

    /// The local position of the cube within the block.
    local_pos: Vec3,

    /// The size of the cube.
    size: Vec3,

    /// The occlusion of this cube.
    occlusion: BlockOcclusion,
}

impl CubeModelBuilder {
    /// Creates a new cube model builder with default settings.
    ///
    /// The default settings for the cube model is a 1x1x1 cube, located at the
    /// origin, with no occlusion.
    pub fn new() -> Self {
        // TODO Add texture atlas support
        Self {
            block_pos: IVec3::ZERO,
            local_pos: Vec3::ZERO,
            size:      Vec3::ONE,
            occlusion: BlockOcclusion::empty(),
        }
    }

    /// Defines the position of this cube model within the block.
    ///
    /// The cube position is relative to the minimum corner of the cube.
    pub fn set_pos(mut self, pos: Vec3) -> Self {
        self.local_pos = pos;
        self
    }

    /// Sets the size of this cube model.
    pub fn set_size(mut self, size: Vec3) -> Self {
        self.size = size;
        self
    }
}

impl Default for CubeModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockModelGenerator for CubeModelBuilder {
    fn write_to_mesh(&self, mesh: &mut TempMesh) {
        let pos = self.block_pos.as_vec3() + self.local_pos;
        let size = self.size;
        let occlusion = self.occlusion;

        let mut quad = |offset: usize| {
            let vertex_count = mesh.vertices.len() as u16;
            mesh.indices
                .extend_from_slice(&QUAD_INDICES.map(|i| i + vertex_count));

            for vert_data in CUBE_VERTICES.iter().skip(offset).take(4) {
                let (vertex, normal, uv) = *vert_data;
                mesh.vertices.push(vertex * size + pos);
                mesh.normals.push(normal);
                mesh.uvs.push(uv);
            }
        };

        if !occlusion.contains(BlockOcclusion::NEG_X) {
            quad(0);
        }

        if !occlusion.contains(BlockOcclusion::POS_X) {
            quad(4);
        }

        if !occlusion.contains(BlockOcclusion::NEG_Y) {
            quad(8);
        }

        if !occlusion.contains(BlockOcclusion::POS_Y) {
            quad(12);
        }

        if !occlusion.contains(BlockOcclusion::NEG_Z) {
            quad(16);
        }

        if !occlusion.contains(BlockOcclusion::POS_Z) {
            quad(20);
        }
    }

    fn set_block_pos(&mut self, pos: IVec3) {
        self.block_pos = pos;
    }

    fn set_occlusion(&mut self, occlusion: BlockOcclusion) {
        self.occlusion = occlusion;
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn half_slab() {
        let mut mesh = TempMesh::default();
        let mut cube = CubeModelBuilder::new().set_size(Vec3::new(1.0, 0.5, 1.0));

        cube.set_block_pos(IVec3::new(3, 7, 2));
        cube.set_occlusion(BlockOcclusion::NEG_Y);
        cube.write_to_mesh(&mut mesh);

        #[rustfmt::skip]
        assert_eq!(mesh.vertices, vec![
            // -X
            Vec3::new(3.0, 7.0, 2.0), Vec3::new(3.0, 7.0, 3.0),
            Vec3::new(3.0, 7.5, 3.0), Vec3::new(3.0, 7.5, 2.0),
            // +X
            Vec3::new(4.0, 7.0, 2.0), Vec3::new(4.0, 7.5, 2.0),
            Vec3::new(4.0, 7.5, 3.0), Vec3::new(4.0, 7.0, 3.0),
            // +Y
            Vec3::new(3.0, 7.5, 2.0), Vec3::new(3.0, 7.5, 3.0),
            Vec3::new(4.0, 7.5, 3.0), Vec3::new(4.0, 7.5, 2.0),
            // -Z
            Vec3::new(3.0, 7.0, 2.0), Vec3::new(3.0, 7.5, 2.0),
            Vec3::new(4.0, 7.5, 2.0), Vec3::new(4.0, 7.0, 2.0),
            // +Z
            Vec3::new(3.0, 7.0, 3.0), Vec3::new(4.0, 7.0, 3.0),
            Vec3::new(4.0, 7.5, 3.0), Vec3::new(3.0, 7.5, 3.0),
        ]);

        #[rustfmt::skip]
        assert_eq!(mesh.normals, vec![
            // -X
            Vec3::new(-1., 0.0, 0.0), Vec3::new(-1., 0.0, 0.0),
            Vec3::new(-1., 0.0, 0.0), Vec3::new(-1., 0.0, 0.0),
            // +X
            Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0),
            // +Y
            Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 1.0, 0.0),
            // -Z
            Vec3::new(0.0, 0.0, -1.), Vec3::new(0.0, 0.0, -1.),
            Vec3::new(0.0, 0.0, -1.), Vec3::new(0.0, 0.0, -1.),
            // +Z
            Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0),
        ]);

        #[rustfmt::skip]
        assert_eq!(mesh.uvs, vec![
            // -X
            Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0),
            // +X
            Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0),
            // +Y
            Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0),
            // -Z
            Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0),
            // +Z
            Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0), Vec2::new(1.0, 0.0),
        ]);

        #[rustfmt::skip]
        assert_eq!(mesh.indices, vec![
            // -X
             0,  1,  2,  0,  2,  3,
            // +X
             4,  5,  6,  4,  6,  7,
            // +Y
             8,  9, 10,  8, 10, 11,
            // -Z
            12, 13, 14, 12, 14, 15,
            // +Z
            16, 17, 18, 16, 18, 19,
        ]);
    }
}
