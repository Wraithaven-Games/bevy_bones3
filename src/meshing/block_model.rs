//! Defines how a block model should be added to a chunk mesh.

use anyhow::{bail, Result};
use bevy::prelude::{IVec3, Mesh, Vec2, Vec3};
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bitflags::bitflags;

use crate::storage::BlockData;

bitflags! {
    /// A bitflag-based enum that defines how a block is currently being occluded.
    pub struct BlockOcclusion: u8 {
        /// If true, the block is occluded in the negative X direction.
        const NEG_X = 0b00000001;

        /// If true, the block is occluded in the positive X direction.
        const POS_X = 0b00000010;

        /// If true, the block is occluded in the negative Y direction.
        const NEG_Y = 0b00000100;

        /// If true, the block is occluded in the positive Y direction.
        const POS_Y = 0b00001000;

        /// If true, the block is occluded in the negative Z direction.
        const NEG_Z = 0b00010000;

        /// If true, the block is occluded in the positive Z direction.
        const POS_Z = 0b00100000;
    }
}

impl BlockOcclusion {
    /// Converts this block occlusion value into a directional offset vector.
    pub fn into_offset(self) -> IVec3 {
        let mut offset = IVec3::ZERO;

        if self.contains(BlockOcclusion::NEG_X) {
            offset += IVec3::NEG_X;
        }

        if self.contains(BlockOcclusion::POS_X) {
            offset += IVec3::X;
        }

        if self.contains(BlockOcclusion::NEG_Y) {
            offset += IVec3::NEG_Y;
        }

        if self.contains(BlockOcclusion::POS_Y) {
            offset += IVec3::Y;
        }

        if self.contains(BlockOcclusion::NEG_Z) {
            offset += IVec3::NEG_Z;
        }

        if self.contains(BlockOcclusion::POS_Z) {
            offset += IVec3::Z;
        }

        offset
    }

    /// Gets the opposite facing value for this block occlusion.
    ///
    /// For a positive value along an axis, this function will return the
    /// negative value of that axis. Likewise, negative value will return
    /// the positive counter parts. This effect is applied for all defined
    /// directional values.
    pub fn opposite_face(self) -> BlockOcclusion {
        let mut value = BlockOcclusion::empty();

        if self.contains(BlockOcclusion::NEG_X) {
            value |= BlockOcclusion::POS_X;
        }

        if self.contains(BlockOcclusion::POS_X) {
            value |= BlockOcclusion::NEG_X;
        }

        if self.contains(BlockOcclusion::NEG_Y) {
            value |= BlockOcclusion::POS_Y;
        }

        if self.contains(BlockOcclusion::POS_Y) {
            value |= BlockOcclusion::NEG_Y;
        }

        if self.contains(BlockOcclusion::NEG_Z) {
            value |= BlockOcclusion::POS_Z;
        }

        if self.contains(BlockOcclusion::POS_Z) {
            value |= BlockOcclusion::NEG_Z;
        }

        value
    }
}

/// Acts as a temporary storage devices for mesh data that can be written to an
/// actual Bevy mesh upon completion.
///
/// The data in this temp mesh is stored in a vector list format, where new
/// vertex data can be written in at any time.
///
/// *NOTE*: The data stored within this temporary mesh is not validated in
/// anyway. Writing the data to a Bevy mesh, however, does offer some type
/// validation, as provided by the Bevy mesh implementation.
#[derive(Debug, Clone, Default)]
pub struct TempMesh {
    /// The vertex positions that make up the mesh.
    pub vertices: Vec<Vec3>,

    /// The vertex normals that make up the mesh.
    pub normals: Vec<Vec3>,

    /// The vertex texture coordinates that make up the mesh.
    pub uvs: Vec<Vec2>,

    /// The mesh indices that describe the triangle layout.
    pub indices: Vec<u16>,
}

impl TempMesh {
    /// Contains this temporary mesh into a Bevy mesh.
    ///
    /// The resulting mesh is laid out using a triangle list topology. This
    /// method returns an error if this temporary mesh data is empty.
    pub fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        self.write_to_mesh(&mut mesh).unwrap();
        mesh
    }

    /// Writes the data from this temporary mesh into an existing Bevy mesh.
    ///
    /// This method returns an error if the provided mesh does not use the
    /// triangle list primitive topology, or if this temporary mesh data is
    /// empty.
    pub fn write_to_mesh(self, mesh: &mut Mesh) -> Result<()> {
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            bail!("Mesh does not use a triangle list topology");
        }

        if self.is_empty() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0; 3]; 0]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0; 3]; 0]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0; 2]; 0]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, vec![[0.0; 4]; 0]);
            mesh.set_indices(None);
            mesh.compute_aabb();
            return Ok(());
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.set_indices(Some(Indices::U16(self.indices)));
        mesh.compute_aabb();
        mesh.generate_tangents().unwrap();

        Ok(())
    }

    /// Checks if this temporary mesh is empty or not.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// A generator for creating a block model that can be written to a temporary
/// chunk mesh.
pub trait BlockModelGenerator {
    /// Sets the position of the block that is being generated.
    fn set_block_pos(&mut self, pos: IVec3);

    /// Sets the block occlusion flags for this block model.
    fn set_occlusion(&mut self, occlusion: BlockOcclusion);

    /// Writes the block model to the provided temporary chunk mesh.
    fn write_to_mesh(&self, mesh: &mut TempMesh);
}

/// A trait that can be defined for a block data object in order to specify how
/// a block model should be generated and added to the chunk mesh.
pub trait BlockShape: BlockData {
    /// Creates a new block shape generator instance.
    ///
    /// This is called for every block within a chunk in order to generate a
    /// corresponding block model to append to the chunk mesh.
    fn get_generator(&self) -> Option<Box<dyn BlockModelGenerator>>;

    /// Gets the neighboring block faces that are occluded by this block shape.
    fn get_occludes(&self) -> BlockOcclusion;
}
