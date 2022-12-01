//! A region defines a cuboid boundary of blocks along a uniform, 3D grid.

use std::fmt::Display;

use anyhow::{bail, Result};
use bevy::prelude::*;

use super::iterators::CuboidIterator;

/// A cuboid region defining a collection of elements within a 3D grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Region {
    /// The position of the region.
    pos: IVec3,

    /// The size of the region.
    size: IVec3,
}

impl Region {
    /// A region that contains a single chunk located at the position (0, 0, 0).
    pub const CHUNK: Region = Region {
        pos:  IVec3::ZERO,
        size: IVec3::new(16, 16, 16),
    };

    /// Creates a new region from two points within the grid.
    ///
    /// Each point is an opposite corner of the grid.
    pub fn from_points(a: IVec3, b: IVec3) -> Self {
        let min = a.min(b);
        let max = a.max(b);
        let size = max - min + 1;

        Self {
            pos: min,
            size,
        }
    }

    /// Creates a new region from a position on the grid and a size.
    ///
    /// The position is the lowest point along the X, Y, and Z axis'.
    ///
    /// This function panics if the size is <= 0 along any axis.
    pub fn from_size(pos: IVec3, size: IVec3) -> Self {
        if size.x <= 0 || size.y <= 0 || size.z <= 0 {
            panic!("Cannot a region with a size <= 0. Found: {size}");
        }

        Self {
            pos,
            size,
        }
    }

    /// Gets the minimum corner of this region.
    pub fn min(&self) -> IVec3 {
        self.pos
    }

    /// Gets the maximum corner of this region.
    pub fn max(&self) -> IVec3 {
        self.pos + self.size - 1
    }

    /// Gets the size of this region.
    pub fn size(&self) -> IVec3 {
        self.size
    }

    /// Gets whether or not the given point is within this region.
    pub fn contains(&self, point: IVec3) -> bool {
        let p = point - self.pos;

        p.x >= 0
            && p.y >= 0
            && p.z >= 0
            && p.x < self.size.x
            && p.y < self.size.y
            && p.z < self.size.z
    }

    /// Contains a position within this region into a unique array index.
    ///
    /// If the given point is not within this region, an error is returned.
    pub fn point_to_index(&self, point: IVec3) -> Result<usize> {
        if !self.contains(point) {
            bail!("Point is outside of region: {point}, Region: {self}");
        }

        let p = point - self.pos;
        let index = p.x * self.size.y * self.size.z + p.y * self.size.z + p.z;
        Ok(index as usize)
    }

    /// Creates a new cuboid iterator over this region.
    pub fn iter(&self) -> CuboidIterator {
        CuboidIterator::from(self)
    }

    /// Gets the number of elements within this region.
    pub fn count(&self) -> usize {
        (self.size.x * self.size.y * self.size.z) as usize
    }
}

impl IntoIterator for Region {
    type IntoIter = CuboidIterator;
    type Item = IVec3;

    fn into_iter(self) -> Self::IntoIter {
        CuboidIterator::from(&self)
    }
}

impl Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Pos: {}, Size: {})", self.pos, self.size)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn index_is_unique() {
        let a = IVec3::new(-17, 2, -3);
        let b = IVec3::new(-20, 4, -2);
        let region = Region::from_points(a, b);

        let mut indices: Vec<usize> =
            region.iter().map(|pos| region.point_to_index(pos).unwrap()).collect();

        indices.dedup();

        assert_eq!(indices.len(), region.count());
        assert_eq!(indices.iter().min(), Some(0).as_ref());
        assert_eq!(indices.iter().max(), Some(region.count() - 1).as_ref());
    }
}
