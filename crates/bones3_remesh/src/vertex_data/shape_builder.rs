//! A utility for preparing vertex data for a set of chunk meshes.

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

use crate::prelude::{BlockModelGenerator, BlockOcclusion};

/// Acts as a temporary storage devices for mesh data that can be written to an
/// actual Bevy mesh upon completion.
#[derive(Debug, Default)]
pub struct TempMesh {
    /// The vertex positions that make up the mesh.
    pub vertices: Vec<Vec3>,

    /// The vertex normals that make up the mesh.
    pub normals: Vec<Vec3>,

    /// The vertex texture coordinates that make up the mesh.
    pub uvs: Vec<Vec2>,

    /// The mesh indices that describe the triangle layout.
    pub indices: Vec<u16>,

    /// The material that is being used for this temporary mesh.
    pub material: Handle<StandardMaterial>,
}

impl TempMesh {
    /// Contains this temporary mesh into a Bevy mesh.
    ///
    /// The resulting mesh is laid out using a triangle list topology. This
    /// method returns an error if this temporary mesh data is empty.
    pub fn into_mesh(self) -> Option<(Mesh, Handle<StandardMaterial>)> {
        if self.indices.is_empty() {
            return None;
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.set_indices(Some(Indices::U16(self.indices)));
        mesh.compute_aabb();
        mesh.generate_tangents().unwrap();

        Some((mesh, self.material))
    }
}

/// A temporary builder object that allows for block model shapes to be
/// constructed in order to build a set of chunk meshes and corresponding
/// material handles.
#[derive(Default)]
pub struct ShapeBuilder {
    /// A list of temporary chunk meshes that will be created.
    meshes: Vec<TempMesh>,

    /// The local position of the block currently being handled.
    local_pos: IVec3,

    /// The current occlusion flags for the block currently being handled.
    occlusion: BlockOcclusion,
}

impl ShapeBuilder {
    /// Gets the position of the block currently being built.
    ///
    /// This value is treated as an offset that is provided to the block model
    /// generator when adding new models to this shape builder instance.
    pub fn get_local_pos(&self) -> IVec3 {
        self.local_pos
    }

    /// Sets the position of the block currently being modified.
    ///
    /// See [`get_local_pos`] for more information.
    pub fn set_local_pos(&mut self, pos: IVec3) {
        self.local_pos = pos;
    }

    /// Gets the current occlusion values applied to the block being handled.
    /// All flags in this bitflag mask represent faces of the block model
    /// that are currently being occluded by other blocks, and should not be
    /// included in the generated block model.
    pub fn get_occlusion(&self) -> BlockOcclusion {
        self.occlusion
    }

    /// Sets the occlusion flags for the block currently being handled.
    ///
    /// See [`get_occlusion`] for more information.
    pub fn set_occlusion(&mut self, occlusion: BlockOcclusion) {
        self.occlusion = occlusion;
    }

    /// Appends a new shape to this shape builder instance with the given
    /// material, based off the provided block model generator.
    pub fn add_shape<G>(&mut self, mut shape: G, material: Handle<StandardMaterial>)
    where
        G: BlockModelGenerator,
    {
        shape.set_block_pos(self.get_local_pos());

        let mesh = match self
            .meshes
            .iter_mut()
            .find(|mesh| mesh.material == material)
        {
            Some(mesh) => mesh,
            None => {
                self.meshes.push(TempMesh {
                    material,
                    ..default()
                });
                self.meshes.last_mut().unwrap()
            },
        };

        shape.write_to_mesh(mesh);
    }

    /// Converts this shape builder into an iterator over all temporary meshes
    /// that need to be created from this shape builder.
    pub fn into_meshes(self) -> impl Iterator<Item = (Mesh, Handle<StandardMaterial>)> {
        self.meshes.into_iter().flat_map(|mesh| mesh.into_mesh())
    }
}
