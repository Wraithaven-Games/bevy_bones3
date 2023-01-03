//! Defines how a block's collision should be handled.

use super::BlockShapeBuilder;
use crate::prelude::BlockData;

/// An extension trait for block data types to add support for block collision
/// handling.
pub trait BlockCollision: BlockData {
    /// Builds the collision shape for this block data.
    ///
    /// This function is called each time the collision for a chunk is rebuilt.
    fn build_collision_shape<'a>(&self, builder: &'a mut BlockShapeBuilder<'a>);
}
