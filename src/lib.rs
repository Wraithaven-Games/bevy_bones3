//! This cargo crate, `bevy_bones3` is a plugin for Bevy that adds support for
//! managing voxel environments. This includes voxel data storage, chunk loading
//! and unloading, and mesh generation.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::invalid_html_tags)]


pub mod math;


/// Contains a re-export of all components and systems defined within this
/// crate.
pub mod prelude {
    pub use super::math::*;
}
