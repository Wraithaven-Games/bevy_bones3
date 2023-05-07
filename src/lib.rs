//! This cargo crate, `bevy_bones3` is a plugin for Bevy that adds support for
//! managing voxel environments. This includes voxel data storage, chunk loading
//! and unloading, and mesh generation.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::invalid_html_tags)]

pub use bones3_core as core;
#[cfg(feature = "meshing")]
pub use bones3_remesh as remesh;
#[cfg(feature = "worldgen")]
pub use bones3_worldgen as worldgen;

/// Used to import common components and systems for Bones Cubed.
pub mod prelude {
    pub use super::core::prelude::*;
    #[cfg(feature = "meshing")]
    pub use super::remesh::prelude::*;
    #[cfg(feature = "worldgen")]
    pub use super::worldgen::prelude::*;
}
