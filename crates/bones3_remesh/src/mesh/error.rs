//! A set of common errors that may be thrown while working with chunk meshing.

use bevy::render::render_resource::PrimitiveTopology;
use thiserror::Error;

/// A set of common errors that may be thrown while working with chunk meshing.
#[derive(Debug, Error)]
pub enum RemeshError {
    /// An error that is thrown when attempting to write vertex data to a mesh
    /// with incorrect topology.
    #[error("Incorrect mesh topology. Expected: TriangleList, found: {0:?}")]
    WrongTopology(PrimitiveTopology),
}
