//! This module contains the Bevy component that implements the chunk anchor.

use bevy::prelude::*;

use crate::prelude::Region;

/// Defines an anchor within a world that forces a radius of chunks around
/// itself to stay loaded.
///
/// This component relies on the Position component in order to function.
#[derive(Debug, Clone, Reflect, FromReflect, Component, Default)]
#[reflect(Component)]
pub struct ChunkAnchor {
    /// The world that this chunk anchor is pinned to.
    world: Option<Entity>,

    /// The radius, in chunks, that are triggered to load around the chunk
    /// anchor.
    ///
    /// A value of 0 will only trigger a single chunk to remain loaded.
    radius: u16,

    /// The maximum number of chunks around this anchor that are allowed to
    /// remain loaded before being considered out of range.
    ///
    /// A value of 0 will only allow for a single chunk to be considered within
    /// range of this anchor.
    max_radius: u32,

    /// The weighted directional value to apply to this anchor.
    ///
    /// This allows for a loading bias to be applied to the chunk loader to
    /// prioritize loading chunks in a specific direction.
    weighted_dir: Vec3,

    /// The last location that the chunk anchor [`iter`] function was called.
    /// This is to present redundant iterators from being triggered if the
    /// chunk anchor has not moved.
    last_center: Option<IVec3>,
}

impl ChunkAnchor {
    /// Creates a new chunk anchor instance with the given chunk load radius
    /// (See [`get_radius`]) and the memory storage radius (See
    /// [`get_max_radius`]).
    ///
    /// If the max radius is less than the radius, then the chunk load radius
    /// will be used instead for the memory storage radius.
    pub fn new(world: Entity, radius: u16, mut max_radius: u32) -> Self {
        if max_radius < radius as u32 {
            max_radius = radius as u32;
        }

        Self {
            world: Some(world),
            radius,
            max_radius,
            weighted_dir: Vec3::ZERO,
            last_center: None,
        }
    }

    /// Gets the entity of the world that this chunk anchor is targeting.
    pub fn get_world(&self) -> Option<Entity> {
        self.world
    }

    /// Sets the world that this chunk anchor should target, if any.
    ///
    /// Calling this function will clear the chunk loading cache for this
    /// anchor.
    pub fn set_world(&mut self, world: Option<Entity>) {
        self.world = world;
        self.last_center = None;
    }

    /// Gets the load radius of this chunk anchor.
    ///
    /// All chunks within this radius will constantly attempt to be loaded if
    /// not already loaded.
    pub fn get_radius(&self) -> u16 {
        self.radius
    }

    /// Gets the memory storage radius of this chunk anchor.
    ///
    /// This radius value does not load chunks but instead tells the system how
    /// many chunks to keep in memory if they are already loaded. A value of
    /// 128 will specify that all chunks within 128 chunks of this anchor
    /// will remain loading, as long as they are loaded by another anchor.
    pub fn get_max_radius(&self) -> u32 {
        self.max_radius
    }

    /// Sets the chunk load radius of this chunk anchor to a new value. (See
    /// [`get_radius`])
    ///
    /// If the new radius is greater than the max radius value, then the max
    /// radius value will be updated to be equal to the new chunk load
    /// radius.
    ///
    /// Updating this value will clear the loading cache attached to this
    /// anchor.
    pub fn set_radius(&mut self, radius: u16) {
        self.radius = radius;
        self.last_center = None;

        if self.max_radius < self.radius as u32 {
            self.max_radius = self.radius as u32;
        }
    }

    /// Sets the memory storage radius of this chunk anchor to a new value. (See
    /// [`get_max_radius`])
    ///
    /// If the new max_radius value is less than the current radius value, then
    /// the current radius value will be used instead.
    pub fn set_max_radius(&mut self, max_radius: u32) {
        if max_radius < self.radius as u32 {
            self.max_radius = self.radius as u32;
        } else {
            self.max_radius = max_radius;
        }
    }

    /// Updates the directional weighting of chunks to prioritize chunks closer
    /// to the direction of the given weight vector. The priority value is based
    /// off the dot product between the weighted direction vector provided and
    /// the normalized directional vector between the anchor position and
    /// the chunk that needs to be loaded.
    ///
    /// A higher magnitude on the weighted directional vector will put a higher
    /// priority bias on the indicated chunks. A common example of this in
    /// action might be to prioritize loading chunk in the direction an
    /// anchor is moving based off it's current velocity, in order to ensure
    /// that an anchor always stays within loaded chunks. Another example
    /// might be to add a weighted direction based off the direction the
    /// camera is facing to prioritize loading chunks that the
    /// player is looking at.
    ///
    /// Calling this function will recalculate the priority value of all chunk
    /// positions the next time the cache is cleared. Calling this function
    /// has no overhead, and will only take affect the next time the cache
    /// is recalculated.
    pub fn set_weighted_dir(&mut self, weighted_dir: Vec3) {
        self.weighted_dir = weighted_dir;
    }

    /// Creates an iterator over all chunks that should be loaded from this
    /// chunk anchor.
    ///
    /// The chunk anchor's current location must be specified.
    ///
    /// Note that this function does cause some allocations based on the
    /// internal sorting of chunk loading order and priority. This function will
    /// also mark the chunk anchor as up-to-date, and will cause all future
    /// iterator calls to return empty until the chunk anchor has moved to a
    /// new chunk.
    pub fn iter(&mut self, center: IVec3) -> impl Iterator<Item = IVec3> + '_ {
        let radius = self.radius as i32;
        Region::from_points(center - radius, center + radius).into_iter()
    }

    /// Gets the chunk priority of the target chunk.
    ///
    /// This function calculates what the priority value of loading the target
    /// would be based on the position of the target and the current status of
    /// this chunk anchor. If the target is outside of the load radius of
    /// this chunk anchor, the returned value is infinity.
    pub fn get_priority(&self, pos: IVec3, target: IVec3) -> f32 {
        let distance = target.as_vec3().distance(pos.as_vec3());
        if distance > self.radius as f32 {
            return f32::INFINITY;
        }

        let view_dir = (target - pos).as_vec3().normalize();
        let weight = view_dir.dot(self.weighted_dir);
        distance - weight
    }
}
