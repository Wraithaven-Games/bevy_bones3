//! This module contains the systems for generating chunk visual meshes based on
//! block shape data.
//!
//! Note that the use of elements within this module require the `meshing`
//! feature of this crate to be enabled.

mod block_model;
mod builder;
mod mesher;
mod vertex_data;

pub use block_model::*;
pub use mesher::*;
pub use vertex_data::*;
