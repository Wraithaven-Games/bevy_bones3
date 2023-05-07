//! Defines how a block model should be added to a chunk mesh.

use bevy::prelude::*;
use bitflags::bitflags;
use bones3_core::prelude::*;

use crate::vertex_data::{ShapeBuilder, TempMesh};

bitflags! {
    #[derive(Copy, Clone)]
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

impl Default for BlockOcclusion {
    fn default() -> Self {
        BlockOcclusion::empty()
    }
}

/// A generator for creating a block model that can be written to a temporary
/// chunk mesh.
pub trait BlockModelGenerator {
    /// Writes the block model to the provided temporary chunk mesh.
    fn write_to_mesh(&self, mesh: &mut TempMesh, pos: IVec3);
}

/// A trait that can be defined for a block data object in order to specify how
/// a block model should be generated and added to the chunk mesh.
pub trait BlockShape: BlockData {
    /// Writes an instance of this block shape to the provided shape builder,
    ///
    /// Information such as the current block occlusion may be retrieved from
    /// the shape builder as needed.
    fn write_shape(&self, shape_builder: &mut ShapeBuilder);

    /// Checks if one tile is to occlude another tile. Returns True if face is
    /// occluded.
    fn check_occlude(&self, face: BlockOcclusion, other: Self) -> bool;
}
