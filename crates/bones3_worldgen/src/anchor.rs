//! This module contains the Bevy component that implements the chunk anchor.

use bevy::prelude::*;

use crate::prelude::Region;

/// Defines an anchor within a world that forces a radius of chunks around
/// itself to stay loaded.
///
/// This component relies on the Position component in order to function.
#[derive(Debug, Clone, Reflect, Component)]
pub struct ChunkAnchor {
    /// The world that this chunk anchor is pinned to.
    world: Entity,

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

    /// The radius around this chunk anchor that are forcefully loaded before
    /// the next frame if not already loaded.
    force_radius: i32,

    /// The weighted directional value to apply to this anchor.
    ///
    /// This allows for a loading bias to be applied to the chunk loader to
    /// prioritize loading chunks in a specific direction.
    weighted_dir: Vec3,
}

impl ChunkAnchor {
    /// Creates a new chunk anchor instance with the given chunk load radius
    /// (See [`get_radius`]), memory storage radius (See
    /// [`get_max_radius`]), and force load radius (See [`get_force_radius`]).
    ///
    /// If the max radius is less than the radius, then the chunk load radius
    /// will be used instead for the memory storage radius.
    pub fn new(world: Entity, radius: u16, mut max_radius: u32, force_radius: i32) -> Self {
        if max_radius < radius as u32 {
            max_radius = radius as u32;
        }

        Self {
            world,
            radius,
            max_radius,
            force_radius,
            weighted_dir: Vec3::ZERO,
        }
    }

    /// Gets the entity of the world that this chunk anchor is targeting.
    pub fn get_world(&self) -> Entity {
        self.world
    }

    /// Sets the world that this chunk anchor should target.
    pub fn set_world(&mut self, world: Entity) {
        self.world = world;
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

    /// Gets the force-load radius of this chunk anchor.
    ///
    /// The force load radius is an emergency radius value that causes all
    /// chunks within this radius around the chunk anchor to immediately be
    /// loaded before the next frame.
    ///
    /// This value may be negative to indicate that there is no force radius for
    /// this chunk anchor.
    pub fn get_force_radius(&self) -> i32 {
        self.force_radius
    }

    /// Sets the chunk load radius of this chunk anchor to a new value. (See
    /// [`get_radius`])
    ///
    /// If the new radius is greater than the max radius value, then the max
    /// radius value will be updated to be equal to the new chunk load
    /// radius.
    pub fn set_radius(&mut self, radius: u16) {
        self.radius = radius;

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

    /// Sets the force radius of this chunk anchor.
    ///
    /// The force load radius is an emergency radius value that causes all
    /// chunks within this radius around the chunk anchor to immediately be
    /// loaded before the next frame.
    ///
    /// This value may be negative to indicate that there is no force radius for
    /// this chunk anchor.
    pub fn set_force_radius(&mut self, force_radius: i32) {
        self.force_radius = force_radius;
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
    pub fn set_weighted_dir(&mut self, weighted_dir: Vec3) {
        self.weighted_dir = weighted_dir;
    }

    /// Creates an iterator over all chunks that should be loaded from this
    /// chunk anchor.
    pub fn iter(&self, center: IVec3) -> impl Iterator<Item = IVec3> + '_ {
        let radius = self.radius as i32;
        Region::from_points(center - radius, center + radius).into_iter()
    }

    /// Creates an iterator over all chunks that should be force loaded from
    /// this chunk anchor.
    ///
    /// If this chunk anchor has a force radius set to a negative value, this
    /// function returns None and does not require any chunks to be force
    /// loaded.
    pub fn iter_force(&self, center: IVec3) -> Option<impl Iterator<Item = IVec3> + '_> {
        let radius = self.force_radius;
        if radius >= 0 {
            Some(Region::from_points(center - radius, center + radius).into_iter())
        } else {
            None
        }
    }

    /// Gets the chunk priority of the target chunk.
    ///
    /// This function calculates what the priority value of loading the target
    /// would be based on the position of the target and the current status of
    /// this chunk anchor. If the target is outside of the load radius of
    /// this chunk anchor, the returned value is negative infinity.
    ///
    /// Targets with a higher priority are more important than targets with a
    /// lower priority value.
    pub fn get_priority(&self, pos: IVec3, target: IVec3) -> f32 {
        let distance = target.as_vec3().distance(pos.as_vec3());
        if distance > self.radius as f32 {
            return f32::NEG_INFINITY;
        }

        let view_dir = (target - pos).as_vec3().normalize();
        let weight = view_dir.dot(self.weighted_dir);
        -distance + weight
    }
}
