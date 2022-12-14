//! Allows the automatic addition of remesh-related components to new chunks.

use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::storage::{VoxelChunk, VoxelWorld};

/// A marker component that is added to worlds. For all new chunks created
/// within worlds with this component, chunks will automatically get mesh
/// handles and similar 3D components for allowing chunk meshes to be generated
/// and rendered.
#[derive(Component, Reflect)]
pub struct RemeshWorld;

/// This system checks for newly created voxel chunks that are children of
/// remesh worlds. As they are discovered, PbrBundles are applied to them to
/// make them ready for remeshing.
pub fn setup_chunk_meshes(
    worlds: Query<(), (With<RemeshWorld>, With<VoxelWorld>)>,
    new_chunks: Query<(Entity, &VoxelChunk), Added<VoxelChunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for (chunk_id, chunk_meta) in new_chunks.iter() {
        let world_id = chunk_meta.world_id();
        if !worlds.contains(world_id) {
            continue;
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0; 3]; 0]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0; 3]; 0]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0; 2]; 0]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, vec![[0.0; 4]; 0]);
        mesh.compute_aabb();

        let material = StandardMaterial::default();
        commands
            .entity(chunk_id)
            .insert((meshes.add(mesh), materials.add(material)));
    }
}
