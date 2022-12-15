//! Contains systems and components that are useful for generating parts of an
//! infinite voxel world on demand as chunks load.
//!
//! This module also handles the automatic loading and unloading of chunks
//! within a voxel world based off a given chunk anchor's position and effect
//! radius.
//!
//! This module requires the `world_gen` feature to use.

mod anchor;
mod generator;
mod prep_chunks;
mod queue;

pub use anchor::*;
pub use generator::*;
pub use prep_chunks::*;
pub use queue::*;
