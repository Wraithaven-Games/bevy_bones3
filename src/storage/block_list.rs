//! This module contains an implementation for reading and writing to a block
//! list component. This allows for more compact world data, as most of the
//! information can be stored within block types.
//!
//! As blocks tend to contain a lot of redundant data within a world, a block
//! list allows for a quick lookup table system to reduce memory overhead of
//! large voxel worlds.

use std::ops::{Index, IndexMut};

use bevy::prelude::*;

use super::Block;

/// A BlockList is a component that contains a list of block type definitions
/// that can be referenced using block IDs in order to easily store redundant
/// information about block handling within a given voxel environment.
#[derive(Debug, Component)]
pub struct BlockList {
    /// The indexed block type list.
    blocks: Vec<Block>,
}

impl BlockList {
    /// Adds a new block type to this block list and returns the ID of that new
    /// block.
    ///
    /// Note that this method does not check for duplicate block types.
    pub fn add_block_type(&mut self, block: Block) -> BlockId {
        self.blocks.push(block);
        BlockId(self.blocks.len() - 1)
    }

    /// Gets the default block within this block list.
    ///
    /// The default block is the block that is used to fill and spaces within a
    /// voxel world that are unspecified, such as when loading new terrain.
    ///
    /// This is usually an air block in most use-cases.
    pub fn default_block(&self) -> &Block {
        &self.blocks[0]
    }

    /// Gets a mutable reference to the default block within this block list.
    ///
    /// The default block is the block that is used to fill and spaces within a
    /// voxel world that are unspecified, such as when loading new terrain.
    ///
    /// This is usually an air block in most use-cases.
    pub fn default_block_mut(&mut self) -> &mut Block {
        &mut self.blocks[0]
    }

    /// Gets the BlockId of the default block.
    pub fn default_block_id() -> BlockId {
        BlockId(0)
    }
}

impl Default for BlockList {
    fn default() -> Self {
        Self {
            blocks: vec![Block::default()],
        }
    }
}

impl Index<BlockId> for BlockList {
    type Output = Block;

    fn index(&self, index: BlockId) -> &Self::Output {
        &self.blocks[index.0]
    }
}

impl IndexMut<BlockId> for BlockList {
    fn index_mut(&mut self, index: BlockId) -> &mut Self::Output {
        &mut self.blocks[index.0]
    }
}

/// A block Id that points to the location of the block data within a block
/// list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub(super) usize);
