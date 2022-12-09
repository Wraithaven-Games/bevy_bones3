//! Contains systems and components that are useful for generating parts of an
//! infinite voxel world on demand as chunks load.
//!
//! This module also handles the automatic loading and unloading of chunks
//! within a voxel world based off a given chunk anchor's position and effect
//! radius.

mod anchor;
mod generator;
mod queue;

pub use anchor::ChunkAnchor;
pub use generator::{WorldGenerator, WorldGeneratorHandler};
pub use queue::{finish_chunk_loading, load_chunks_async};
