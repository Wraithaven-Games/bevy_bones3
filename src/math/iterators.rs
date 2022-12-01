//! A collection of useful coordinate iterators.

use bevy::prelude::*;

use super::region::Region;

/// An iterator for a cuboid grid of coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuboidIterator {
    /// The minimum corner point.
    min: IVec3,

    /// The maximum corner point.
    max: IVec3,

    /// The next coordinate value within the iterator.
    next: Option<IVec3>,
}

impl CuboidIterator {
    /// Creates a new cuboid iterator from two opposite corner points.
    pub fn from(region: &Region) -> Self {
        Self {
            min:  region.min(),
            max:  region.max(),
            next: Some(region.min()),
        }
    }
}

impl Iterator for CuboidIterator {
    type Item = IVec3;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next {
            let mut value = next;

            value.z += 1;
            if value.z > self.max.z {
                value.z = self.min.z;
                value.y += 1;

                if value.y > self.max.y {
                    value.y = self.min.y;
                    value.x += 1;

                    if value.x > self.max.x {
                        self.next = None;
                    } else {
                        self.next = Some(value);
                    }
                } else {
                    self.next = Some(value);
                }
            } else {
                self.next = Some(value);
            }

            Some(next)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_cuboid() {
        let a = IVec3::new(-1, 0, 3);
        let b = IVec3::new(0, 0, 2);
        let mut iter = CuboidIterator::from(&Region::from_points(a, b));

        assert_eq!(iter.next(), Some(IVec3::new(-1, 0, 2)));
        assert_eq!(iter.next(), Some(IVec3::new(-1, 0, 3)));
        assert_eq!(iter.next(), Some(IVec3::new(0, 0, 2)));
        assert_eq!(iter.next(), Some(IVec3::new(0, 0, 3)));
        assert_eq!(iter.next(), None);
    }
}
