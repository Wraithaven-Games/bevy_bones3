//! This cargo crate, `bevy_bones3` is a plugin for Bevy that adds support for
//! managing voxel environments. This includes voxel data storage, chunk loading
//! and unloading, and mesh generation.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::invalid_html_tags)]
#![allow(clippy::type_complexity)]

pub mod plugins;
pub use bones3_core as core;

/// Used to import common components and systems for Bones Cubed.
pub mod prelude {
    pub use super::core::prelude::*;
    pub use super::plugins::*;
}
