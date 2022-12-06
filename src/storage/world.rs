//! An infinite grid of voxels.
//!
//! Data within the world is stored as 16x16x16 chunks of data that must be
//! loaded and unloaded as needed to properly manipulate the data within the
//! world.

use anyhow::{anyhow, Result};
use bevy::prelude::*;

use super::chunk::VoxelChunk;
use super::sector::VoxelSector;
use super::voxel::{
    ChunkStorage,
    InitChunkResult,
    UnloadAllChunksResult,
    UnloadChunkResult,
    VoxelStorage,
    VoxelStorageRegion,
};
use super::{BlockData, BlockRegion};
use crate::math::region::Region;

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

impl<T: BlockData> VoxelStorageRegion<T> for VoxelWorld<T> {
    fn get_block_region(&self, region: Region) -> BlockRegion<T> {
        let mut block_region = BlockRegion::new(region);

        let chunks = self
            .sectors
            .iter()
            .filter(|s| Region::SECTOR.shift(s.get_sector_coords() << 8).intersects(region))
            .flat_map(|s| s.chunk_iter())
            .flat_map(|chunk| -> Result<(&VoxelChunk<T>, Region)> {
                Ok((
                    chunk,
                    Region::intersection(
                        &Region::CHUNK.shift(chunk.get_chunk_coords() << 4),
                        &region,
                    )?,
                ))
            });

        for (chunk, region) in chunks {
            for pos in region.iter() {
                let local_pos = pos & 15;
                let block = chunk.get_block(local_pos).unwrap();
                block_region.set_block(pos, block).unwrap();
            }
        }

        block_region
    }
}

impl<T: BlockData> ChunkStorage for VoxelWorld<T> {
    fn init_chunk(&mut self, chunk_coords: IVec3) -> InitChunkResult {
        let sector_coords = chunk_coords >> 4;

        let find_sector = &mut self.sectors.iter_mut() // <br>
            .find(|s| s.get_sector_coords().eq(&sector_coords));

        if let Some(sector) = find_sector {
            sector.init_chunk(chunk_coords)
        } else {
            let mut sector = VoxelSector::new(sector_coords);
            let result = sector.init_chunk(chunk_coords);
            if let InitChunkResult(Ok(_)) = result {
                self.sectors.push(sector);
            }
            result
        }
    }

    fn unload_chunk(&mut self, chunk_coords: IVec3) -> UnloadChunkResult {
        let sector_coords = chunk_coords >> 4;

        let find_sector = &mut self.sectors.iter_mut() // <br>
            .find(|s| s.get_sector_coords().eq(&sector_coords));

        if let Some(sector) = find_sector {
            sector.unload_chunk(chunk_coords)
        } else {
            UnloadChunkResult(Err(anyhow!("Chunk ({}) does not exist", chunk_coords)))
        }
    }

    fn unload_all_chunks(&mut self) -> UnloadAllChunksResult {
        let chunk_list = self.sectors.iter_mut().flat_map(|s| s.unload_all_chunks().0).collect();
        self.sectors.clear();

        UnloadAllChunksResult(chunk_list)
    }

    fn is_chunk_loaded(&self, chunk_coords: IVec3) -> Result<bool> {
        let sector_coords = chunk_coords >> 4;

        let find_sector = &self.sectors.iter().find(|s| s.get_sector_coords().eq(&sector_coords));

        if let Some(sector) = find_sector {
            sector.is_chunk_loaded(chunk_coords)
        } else {
            Ok(false)
        }
    }
}
