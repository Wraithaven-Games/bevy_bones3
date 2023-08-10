//! This module defines the Block data container that describes how a block
//! should be handled by other systems.

use bevy::prelude::*;

/// A block type definition, which describes how a block should be handled and
/// what information a specific block type contains.
#[derive(Debug, Default)]
pub struct Block {
    /// The static model that is used when placing this block within a chunk.
    #[cfg(feature = "meshing")]
    pub(crate) static_model: Option<Handle<Mesh>>,
}
