//! This module contains the resources that may be used to generate chunk meshes
//! and interact with the remesh systems.

use bevy::prelude::*;
use bevy::utils::HashMap;

/// This resource contains an indexed list of material handles that are used by
/// blocks when generating chunk meshes.
#[derive(Resource, Default)]
pub struct ChunkMaterialList {
    /// The indexed list of material handles.
    materials: Vec<Handle<StandardMaterial>>,

    /// Material names and their corresponding index values within the material
    /// list.
    material_keys: HashMap<String, u16>,
}

impl ChunkMaterialList {
    /// Adds a new material to the chunk material list.
    ///
    /// This function returns the index of the newly added material.
    pub fn add_material(
        &mut self,
        material: Handle<StandardMaterial>,
        name: Option<String>,
    ) -> u16 {
        self.materials.push(material);
        let index = (self.materials.len() - 1) as u16;

        if let Some(material_name) = name {
            self.material_keys.insert(material_name, index);
        }

        index
    }

    /// Gets a copy of the material handle at the given material index.
    pub fn get_material(&self, index: u16) -> Handle<StandardMaterial> {
        self.materials[index as usize].clone()
    }

    /// Tries to find a material within this material list with the given name.
    ///
    /// Returns the index of the material, or `None` if the material could not
    /// be found.
    pub fn find_material(&self, name: &str) -> Option<u16> {
        self.material_keys.get(name).copied()
    }
}
