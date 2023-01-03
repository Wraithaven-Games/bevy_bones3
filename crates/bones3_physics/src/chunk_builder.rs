//! Handles the reconstruction of chunk collision shapes.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use ordered_float::OrderedFloat;

use super::BlockCollision;
use crate::math::Region;
use crate::storage::{BlockData, VoxelStorage};

/// A marker component that indicates that a chunk needs it's collision shape
/// regenerated.
#[derive(Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct RebuildChunkCollision;

/// The settings for a new compound collider.
struct CompoundColliderDef {
    /// The friction value for the collider.
    friction: OrderedFloat<f32>,

    /// The restitution value for the collider.
    restitution: OrderedFloat<f32>,

    /// The list of shapes in the compound collider.
    shapes: Vec<(Vect, Rot, Collider)>,
}

/// This system will rebuild the colliders for each chunk where the
/// `RebuildChunkCollider` marker component is defined.
pub fn rebuild_chunk_collision<T>(
    chunks: Query<(Entity, Option<&Children>, &VoxelStorage<T>), With<RebuildChunkCollision>>,
    chunk_colliders: Query<Entity, With<Collider>>,
    mut commands: Commands,
) where
    T: BlockData + BlockCollision,
{
    for (chunk_id, children, blocks) in chunks.iter() {
        if let Some(children) = children {
            for child in children.iter().flat_map(|id| chunk_colliders.get(*id)) {
                commands.entity(child).despawn();
            }
        }

        let mut colliders: Vec<CompoundColliderDef> = vec![];

        for local_pos in Region::CHUNK.iter() {
            let mut builder = BlockShapeBuilder {
                colliders:         &mut colliders,
                block_translation: local_pos.as_vec3(),
            };

            let block = blocks.get_block(local_pos);
            block.build_collision_shape(&mut builder);
        }

        commands
            .entity(chunk_id)
            .remove::<RebuildChunkCollision>()
            .with_children(|parent| {
                for collider_def in colliders {
                    parent.spawn((
                        TransformBundle::default(),
                        Friction::coefficient(*collider_def.friction),
                        Restitution::coefficient(*collider_def.restitution),
                        Collider::compound(collider_def.shapes),
                    ));
                }
            });
    }
}

/// A builder struct for adding collision handles to a block when a chunk
/// collision is being rebuilt.
pub struct BlockShapeBuilder<'a> {
    /// A list of colliders that will be generated.
    colliders: &'a mut Vec<CompoundColliderDef>,

    /// The translation of the block position within the chunk bounds.
    block_translation: Vec3,
}

impl<'a> BlockShapeBuilder<'a> {
    /// Gets or creates a mutable reference to the compound collider definition
    /// with the given friction and restitution values.
    fn get_def(&'a mut self, friction: f32, restitution: f32) -> &'a mut CompoundColliderDef {
        let friction = OrderedFloat(friction);
        let restitution = OrderedFloat(restitution);

        let pos = self
            .colliders
            .iter()
            .position(|def| def.friction == friction && def.restitution == restitution)
            .or_else(|| {
                let col_def = CompoundColliderDef {
                    friction,
                    restitution,
                    shapes: vec![],
                };

                self.colliders.push(col_def);
                Some(self.colliders.len() - 1)
            })
            .unwrap();

        &mut self.colliders[pos]
    }

    /// Begins initialization of a new cuboid block shape with the given block
    /// size.
    pub fn add_cube(&'a mut self, size: Vec3) -> BlockShapeDefinitionBuilder<'a> {
        BlockShapeDefinitionBuilder::new(
            self,
            Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
            self.block_translation,
        )
    }
}

/// A temporary builder object that defines the creation of a new block
/// collision shape element.
pub struct BlockShapeDefinitionBuilder<'a> {
    /// A reference to the master block shape builder.
    builder: &'a mut BlockShapeBuilder<'a>,

    /// The collider of the collision shape.
    collider: Collider,

    /// The translation of the collision shape within the block bounds.
    local_translation: Vec3,

    /// The translation of the block within the chunk bounds.
    block_translation: Vec3,

    /// The rotation of the collision shape.
    rotation: Quat,

    /// The friction of the collision shape.
    friction: f32,

    /// The restitution of the collision shape.
    restitution: f32,
}

impl<'a> BlockShapeDefinitionBuilder<'a> {
    /// Creates a new block shape definition builder with the given collider
    /// instance and all default Rapier3d collider settings.
    fn new(
        builder: &'a mut BlockShapeBuilder<'a>,
        collider: Collider,
        block_translation: Vec3,
    ) -> Self {
        Self {
            builder,
            collider,
            local_translation: Vec3::ZERO,
            block_translation,
            rotation: Quat::IDENTITY,
            friction: 0.5,
            restitution: 0.0,
        }
    }

    /// Sets the translation of this block shape definition within the block
    /// bounds.
    pub fn translation(mut self, translation: Vec3) -> Self {
        self.local_translation = Vect::new(translation.x, translation.y, translation.z);
        self
    }

    /// Sets the rotation of this block shape definition.
    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the friction coefficient of this block shape definition.
    ///
    /// (Default is 0.5)
    pub fn friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    /// Sets the restitution coefficient of this block shape definition.
    ///
    /// (Default is 0.0)
    pub fn restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    /// Finalizes this collision shape element and pushes it to the chunk
    /// collision handler.
    pub fn build(self) {
        self.builder
            .get_def(self.friction, self.restitution)
            .shapes
            .push((
                self.local_translation + self.block_translation,
                self.rotation,
                self.collider,
            ));
    }
}
