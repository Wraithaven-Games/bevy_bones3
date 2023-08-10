//! This module defines events used within the Bones3 storage sub-plugin.

use bevy::prelude::*;

use super::BlockId;

/// This event can be used to change a block within a voxel world.
#[derive(Debug, Event)]
pub struct SetBlockEvent {
    /// The world that contains the block to change.
    pub world: Entity,

    /// The coordinates of the block to change.
    pub coords: IVec3,

    /// The block id to assign to the given block coordinates.
    pub block_id: BlockId,
}
