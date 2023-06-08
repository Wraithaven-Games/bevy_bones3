//! A handler for an abstract chunk anchor component to load and reference
//! chunks based off the anchor's current location.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::prelude::{Region, VoxelChunk, VoxelWorld};

/// This plugin can be used to create a new chunk anchor component for easily
/// querying and prioritizing chunks around the anchor.
#[derive(Default)]
pub struct ChunkAnchorPlugin<T>
where
    T: Send + Sync + Default,
{
    /// Default placeholder for T.
    _phantom: PhantomData<T>,
}

impl<T> Plugin for ChunkAnchorPlugin<T>
where
    T: Send + Sync + Default + 'static,
{
    fn build(&self, app: &mut App) {
        app.register_type::<ChunkAnchor<T>>()
            .register_type::<ChunkAnchorRecipient<T>>()
            .add_system(
                clear_coords_without_transform::<T>
                    .in_base_set(CoreSet::PostUpdate)
                    .in_set(ChunkAnchorSet::UpdateCoords),
            )
            .add_system(
                update_coords::<T>
                    .in_base_set(CoreSet::PostUpdate)
                    .in_set(ChunkAnchorSet::UpdateCoords),
            )
            .add_system(
                update_chunk_priorities::<T>
                    .in_base_set(CoreSet::PostUpdate)
                    .in_set(ChunkAnchorSet::UpdatePriorities),
            )
            .add_system(
                attach_chunk_recipient_comp::<T>
                    .in_base_set(CoreSet::PostUpdate)
                    .in_set(ChunkAnchorSet::AttachChunkComponents),
            )
            .configure_set(ChunkAnchorSet::UpdatePriorities.after(ChunkAnchorSet::UpdateCoords));
    }
}

/// These system sets are used for all chunk anchor plugin handling.
#[derive(Debug, SystemSet, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ChunkAnchorSet {
    /// This system set is used for updating the coordinates of all chunk
    /// anchors.
    UpdateCoords,

    /// This system set is used for updating the priority values of all chunks
    /// based off existing chunk anchors.
    UpdatePriorities,

    /// This system set is a basic utility system for automatically adding
    /// components to chunks for working with chunk anchors.
    AttachChunkComponents,
}

/// A basic chunk anchor component that can be used to process and weight nearby
/// chunks.
///
/// This component should be attached to an entity with a SpatialBundle
/// attached, otherwise it will not perform any actions.
#[derive(Debug, Component, Reflect, Clone)]
pub struct ChunkAnchor<T>
where
    T: Send + Sync,
{
    /// Default placeholder for T.
    #[reflect(ignore)]
    _phantom: PhantomData<T>,

    /// The radius around this chunk anchor that can be processed.
    pub radius: UVec3,

    /// The weight multiplier for this chunk anchor to apply to all nearby chunk
    /// priorities.
    ///
    /// This is useful in situations where some chunk anchors are more important
    /// than others, and need a higher priority.
    ///
    /// Defaults to `1.0`.
    pub weight: f32,

    /// A directional weight bias to apply to this chunk anchor.
    ///
    /// This can be used in situations like prioritizing remeshing chunks in the
    /// direction a player is looking, or prioritizing loading chunks in a
    /// direction a player is moving.
    ///
    /// Set to `(0, 0, 0)` to disable.
    pub dir_bias: Vec3,

    /// The ID of the world this chunk anchor is linked to.
    pub world_id: Entity,

    /// The current coordinates of this chunk anchor relative to the world. This
    /// value is internally updated each frame.
    ///
    /// If the chunk anchor is not attached to an entity with a SpatialBundle,
    /// or the world cannot be accessed, then the coordinates are set to
    /// `None`.
    pub coords: Option<IVec3>,
}

impl<T> ChunkAnchor<T>
where
    T: Send + Sync,
{
    /// Creates a new chunk anchor instance for the given world ID, with the
    /// specified radius. All weights and bias are set to their default values.
    pub fn new(world_id: Entity, radius: UVec3) -> Self {
        Self {
            _phantom: PhantomData::default(),
            radius,
            weight: 1.0,
            dir_bias: Vec3::ZERO,
            world_id,
            coords: None,
        }
    }

    /// Calculates the current priority value of the chunk at the given target
    /// coordinates based off this chunk anchor's current coordinates.
    ///
    /// This value returns `None` if the chunk is out of range, or if this chunk
    /// anchor has not yet calculated its current coordinates.
    pub fn get_priority(&self, target: IVec3) -> Option<f32> {
        let Some(coords) = self.coords else {
            return None;
        };

        let delta = (coords - target).abs().as_uvec3();
        let radius = self.radius;
        if delta.x > radius.x || delta.y > radius.y || delta.z > radius.z {
            return None;
        };

        let a = coords.as_vec3();
        let b = target.as_vec3();

        let distance = a.distance(b);
        let view_dir = (b - a).normalize_or_zero();
        let weight = view_dir.dot(self.dir_bias);
        let priority = (-distance + weight) * self.weight;
        Some(priority)
    }

    /// Gets the region around this chunk anchor that contains all chunks within
    /// this anchor's range.
    ///
    /// If this chunk anchor does not have a defined coordinate location, then
    /// this method returns `None`.
    pub fn get_region(&self) -> Option<Region> {
        let Some(coords) = self.coords else {
            return None;
        };

        let radius = self.radius.as_ivec3();
        Some(Region::from_points(coords - radius, coords + radius))
    }
}

/// This component is attached to new chunks entities and is used to hold the
/// current priority levels as determined by all existing chunk anchors.
#[derive(Debug, Default, Component, Reflect, Clone)]
pub struct ChunkAnchorRecipient<T>
where
    T: Send + Sync + Default,
{
    /// Default placeholder for T.
    #[reflect(ignore)]
    _phantom: PhantomData<T>,

    /// The current priority value of this chunk recipient based on all nearby
    /// chunk anchors. This value is set to `None` if there are currently no
    /// chunk anchors within range.
    ///
    /// This value is updated internally each frame.
    pub priority: Option<f32>,
}

/// This system checks to see if there are any chunk anchors without an attached
/// SpatialBundle. If so, it clears the internal chunk coordinates of that
/// anchor.
pub(crate) fn clear_coords_without_transform<T>(
    mut anchors: Query<&mut ChunkAnchor<T>, Without<GlobalTransform>>,
) where
    T: Send + Sync + 'static,
{
    for mut anchor in anchors.iter_mut() {
        anchor.coords = None;
    }
}

/// This system is called every frame to update the internal chunk coordinates
/// within all chunk anchors, where a value can be calculated.
pub(crate) fn update_coords<T>(
    worlds: Query<&GlobalTransform, With<VoxelWorld>>,
    mut anchors: Query<(&mut ChunkAnchor<T>, &GlobalTransform)>,
) where
    T: Send + Sync + 'static,
{
    anchors
        .par_iter_mut()
        .for_each_mut(|(mut anchor, anchor_transform)| {
            let Ok(world_transform) = worlds.get(anchor.world_id) else {
                anchor.coords = None;
                return;
            };

            anchor.coords = Some(
                anchor_transform
                    .reparented_to(world_transform)
                    .translation
                    .as_ivec3()
                    >> 4,
            );
        });
}

/// This system is called every frame in order to update the current chunk
/// priorities as determined by all nearby chunk anchors.
pub(crate) fn update_chunk_priorities<T>(
    anchors: Query<&ChunkAnchor<T>>,
    mut chunks: Query<(&mut ChunkAnchorRecipient<T>, &VoxelChunk)>,
) where
    T: Send + Sync + Default + 'static,
{
    chunks
        .par_iter_mut()
        .for_each_mut(|(mut anchor_recipient, chunk_meta)| {
            anchor_recipient.priority = None;

            for anchor in anchors.iter() {
                if anchor.world_id != chunk_meta.world_id() {
                    continue;
                }

                let Some(priority) = anchor.get_priority(chunk_meta.chunk_coords()) else {
                    continue;
                };

                anchor_recipient.priority = Some(match anchor_recipient.priority {
                    Some(old_priority) => f32::max(priority, old_priority),
                    None => priority,
                });
            }
        });
}

/// This system automatically adds the `ChunkAnchorRecipient` component to all
/// chunks that have been created without this component already.
pub(crate) fn attach_chunk_recipient_comp<T>(
    new_chunks: Query<Entity, (With<VoxelChunk>, Without<ChunkAnchorRecipient<T>>)>,
    mut commands: Commands,
) where
    T: Send + Sync + Default + 'static,
{
    for chunk_id in new_chunks.iter() {
        commands
            .entity(chunk_id)
            .insert(ChunkAnchorRecipient::<T>::default());
    }
}
