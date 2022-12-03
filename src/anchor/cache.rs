//! This module contains a chunk loader cache implementation to (hopefully) make
//! deciding which chunks to load be faster and have no memory allocations.

use std::cmp::Ordering;

use bevy::prelude::*;

use crate::math::region::Region;

/// An internal cache for the chunk loader.
///
/// Internally, this is a list of sorted chunk coordinate offsets to be applied
/// relative to the chunk anchor position. This cache contains automatic chunk
/// load order sorted.
///
/// The value of R represents the maximum number of offset node positions to
/// store in this cache. Chunk anchors with a load radius higher than the value
/// of R will be reduced to only loaded chunks at a distance of R from the
/// anchor.
///
/// Higher values of R might cause larger memory overhead and lower performance
/// when updating the directional weight.
pub struct ChunkLoaderCache<const R: u8> {
    /// The list of chunk coordinate offset nodes.
    offsets: Vec<ChunkLoaderNode>,
}

impl<const R: u8> ChunkLoaderCache<R> {
    /// Updates the directional weighting of chunks within this cache.
    ///
    /// Calling this function will recalculate the priority value of all nodes
    /// within this cache and re-sort the node list.
    pub fn update_weighted_dir(&mut self, weighted_dir: Vec3) {
        self.offsets
            .iter_mut()
            .for_each(|n| n.priority = n.distance - n.pos.as_vec3().normalize().dot(weighted_dir));

        self.offsets.sort_unstable_by(|a, b| {
            a.priority.partial_cmp(&b.priority).unwrap_or(Ordering::Equal)
        });
    }

    /// Creates a new chunk position iterator within this chunk loader cache for
    /// the given anchor center position and view radius.
    pub fn iter(&self, radius: f32, center: IVec3) -> impl Iterator<Item = IVec3> + '_ {
        self.offsets
            .iter()
            .filter(move |node| node.distance <= radius)
            .map(move |node| node.pos + center)
    }
}

impl<const R: u8> Default for ChunkLoaderCache<R> {
    fn default() -> Self {
        let count = (R as i32 * 2 + 1).pow(4);
        let mut offsets = Vec::with_capacity(count as usize);

        Region::from_points(IVec3::ZERO - R as i32, IVec3::ZERO + R as i32)
            .iter()
            .for_each(|pos| offsets.push(ChunkLoaderNode::new(pos)));

        Self {
            offsets,
        }
    }
}

/// A chunk coordinate offset node, for cache purposes when determining chunk
/// loading order.
#[derive(Debug, Clone)]
struct ChunkLoaderNode {
    /// The offset of the chunk coordinate, relative to the chunk anchor
    /// position.
    pos: IVec3,

    /// The distance from the position of this node to the center.
    distance: f32,

    /// The priority value of this chunk loader node. Nodes with a lower value
    /// will always attempt to be loaded before nodes with a higher value.
    priority: f32,
}

impl ChunkLoaderNode {
    /// Creates a new chunk loader node instance for the given offset position.
    fn new(pos: IVec3) -> Self {
        let distance = pos.as_vec3().length();
        Self {
            pos,
            distance,
            priority: distance,
        }
    }
}
