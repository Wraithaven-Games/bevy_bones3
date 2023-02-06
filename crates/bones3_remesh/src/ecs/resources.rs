//! This module contains the resources that may be used to generate chunk meshes
//! and interact with the remesh systems.

use bevy::prelude::*;

/// This resource contains an indexed list of material handles that are used by
/// blocks when generating chunk meshes.
#[derive(Resource, Default)]
pub struct ChunkMaterialList {
    /// The indexed list of material handles.
    materials: Vec<Handle<StandardMaterial>>,
}

impl ChunkMaterialList {
    /// Adds a new material to the chunk material list.
    ///
    /// This function returns the index of the newly added material.
    pub fn add_material(&mut self, material: Handle<StandardMaterial>) -> u16 {
        self.materials.push(material);
        (self.materials.len() - 1) as u16
    }

    /// Gets a copy of the material handle at the given material index.
    pub fn get_material(&self, index: u16) -> Handle<StandardMaterial> {
        self.materials[index as usize].clone()
    }
}
