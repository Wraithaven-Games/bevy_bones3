//! This cargo crate, `bevy_bones3` is a plugin for Bevy that adds support for
//! managing voxel environments. This includes voxel data storage, chunk loading
//! and unloading, and mesh generation.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::invalid_html_tags)]
#![allow(clippy::type_complexity)]

pub mod math;
#[cfg(feature = "meshing")]
pub mod meshing;
pub mod plugin;
pub mod query;
pub mod storage;
pub mod world_gen;

/// Contains a re-export of all components and systems defined within this
/// crate.
pub mod prelude {
    pub use super::math::*;
    #[cfg(feature = "meshing")]
    pub use super::meshing::*;
    pub use super::plugin::*;
    pub use super::query::*;
    pub use super::storage::*;
    pub use super::world_gen::*;
}
