//! An infinite grid of voxels.
//!
//! Data within the world is stored as 16x16x16 chunks of data that must be
//! loaded and unloaded as needed to properly manipulate the data within the
//! world.


use anyhow::{anyhow, bail, Result};
use bevy::prelude::*;

use super::sector::VoxelSector;
use super::voxel::{ChunkLoad, VoxelStorage};
use super::BlockData;


/// A marker component indicating the parent entity of a voxel world.
#[derive(Debug, Reflect, Component, Default)]
#[reflect(Component)]
pub struct VoxelWorld<T: BlockData> {
    /// A list of all chunk sectors within this world.
    #[reflect(ignore)]
    sectors: Vec<VoxelSector<T>>,
}

impl<T: BlockData> VoxelStorage<T> for VoxelWorld<T> {
    fn get_block(&self, block_coords: IVec3) -> Result<T> {
        let sector_coords = block_coords >> 8;

        let data = self
            .sectors
            .iter()
            .find(|s| s.get_sector_coords().eq(&sector_coords))
            .map_or_else(T::default, |s| s.get_block(block_coords).unwrap());

        Ok(data)
    }

    fn set_block(&mut self, block_coords: IVec3, data: T) -> Result<()> {
        let sector_coords = block_coords >> 8;

        self.sectors
            .iter_mut()
            .find(|s| s.get_sector_coords().eq(&sector_coords))
            .map_or_else(
                || {
                    Err(anyhow!(
                        "Chunk ({}) has not been initialized and cannot be written to",
                        block_coords >> 4,
                    ))
                },
                |s| s.set_block(block_coords, data),
            )
    }
}

impl<T: BlockData> ChunkLoad for VoxelWorld<T> {
    fn init_chunk(&mut self, chunk_coords: IVec3) -> Result<()> {
        let sector_coords = chunk_coords >> 4;

        let find_sector = &mut self.sectors.iter_mut() // <br>
            .find(|s| s.get_sector_coords().eq(&sector_coords));

        if let Some(sector) = find_sector {
            sector.init_chunk(chunk_coords)
        } else {
            let mut sector = VoxelSector::new(sector_coords);
            match sector.init_chunk(chunk_coords) {
                Err(e) => Err(e),
                Ok(_) => {
                    self.sectors.push(sector);
                    Ok(())
                },
            }
        }
    }

    fn unload_chunk(&mut self, chunk_coords: IVec3) -> Result<()> {
        let sector_coords = chunk_coords >> 4;

        let find_sector = &mut self.sectors.iter_mut() // <br>
            .find(|s| s.get_sector_coords().eq(&sector_coords));

        if let Some(sector) = find_sector {
            sector.unload_chunk(chunk_coords)
        } else {
            bail!("Chunk ({}) does not exist", chunk_coords);
        }
    }

    fn is_loaded(&self, chunk_coords: IVec3) -> Result<bool> {
        let sector_coords = chunk_coords >> 4;

        let find_sector = &self.sectors.iter().find(|s| s.get_sector_coords().eq(&sector_coords));

        if let Some(sector) = find_sector {
            sector.is_loaded(chunk_coords)
        } else {
            Ok(false)
        }
    }
}
