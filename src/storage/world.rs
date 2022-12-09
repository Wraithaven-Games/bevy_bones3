//! An infinite grid of voxels.
//!
//! Data within the world is stored as 16x16x16 chunks of data that must be
//! loaded and unloaded as needed to properly manipulate the data within the
//! world.

use anyhow::{anyhow, bail, Result};
use bevy::prelude::*;

use super::events::{InitChunkResult, UnloadAllChunksResult, UnloadChunkResult};
use super::voxel::{ChunkLoadState, ChunkStorage, VoxelStorage, VoxelStorageRegion};
use super::{BlockData, VoxelWorldSlice};
use crate::math::Region;

/// A single 16x16x16 grid of data values that are stored within a voxel chunk.
/// The block data is stored in a fixed array on the heap.
#[derive(Debug)]
struct VoxelChunk<T: BlockData> {
    /// The block data array for this chunk.
    blocks: Box<[T; 4096]>,

    /// The coordinates of this chunk.
    chunk_coords: IVec3,

    /// The current load state of this chunk.
    load_state: ChunkLoadState,
}

impl<T: BlockData> VoxelChunk<T> {
    /// Creates a new voxel chunk at the given chunk coordinates.
    fn new(chunk_coords: IVec3, load_state: ChunkLoadState) -> Self {
        Self {
            blocks: Box::new([default(); 4096]),
            chunk_coords,
            load_state,
        }
    }
}

/// A single 16x16x16 grid of chunks within a voxel world that store a single,
/// specific type of data. These chunks may optionally be defined.
#[derive(Debug)]
struct VoxelSector<T: BlockData> {
    /// The chunk array grid for this sector.
    chunks: Box<[Option<VoxelChunk<T>>; 4096]>,

    /// The coordinates of this sector.
    sector_coords: IVec3,
}

impl<T: BlockData> VoxelSector<T> {
    /// Creates a new voxel sector with the given sector coordinates.
    fn new(sector_coords: IVec3) -> Self {
        Self {
            chunks: Box::new([(); 4096].map(|_| None)),
            sector_coords,
        }
    }
}

/// A marker component indicating the parent entity of a voxel world.
#[derive(Debug, Reflect, Component, Default)]
#[reflect(Component)]
pub struct VoxelWorld<T: BlockData> {
    /// A list of all chunk sectors within this world.
    #[reflect(ignore)]
    sectors: Vec<VoxelSector<T>>,
}

impl<T: BlockData> VoxelStorage<T> for VoxelWorld<T> {
    fn get_block(&self, block_coords: IVec3) -> T {
        let sector_coords = block_coords >> 8;
        let Some(sector) =  self
            .sectors
            .iter()
            .find(|s| s.sector_coords == sector_coords) else {
                return T::default();
            };

        let chunk_coords = (block_coords >> 4) & 15;
        let chunk_index = Region::CHUNK.point_to_index(chunk_coords).unwrap();
        if let Some(chunk) = &sector.chunks[chunk_index] {
            let block_index = Region::CHUNK.point_to_index(block_coords & 15).unwrap();
            chunk.blocks[block_index]
        } else {
            T::default()
        }
    }

    fn set_block(&mut self, block_coords: IVec3, data: T) -> Result<()> {
        let sector_coords = block_coords >> 8;
        let Some(sector) = self.sectors.iter_mut().find(|s| s.sector_coords == sector_coords) else {
            bail!(                        "Chunk ({}) has not been initialized and cannot be written to",
            block_coords >> 4
);
        };

        let chunk_coords = (block_coords >> 4) & 15;
        let chunk_index = Region::CHUNK.point_to_index(chunk_coords).unwrap();
        let Some(chunk) = &mut sector.chunks[chunk_index] else {
            bail!(                        "Chunk ({}) has not been initialized and cannot be written to",
            block_coords >> 4
        );
        };

        let block_index = Region::CHUNK.point_to_index(block_coords & 15).unwrap();
        chunk.blocks[block_index] = data;

        Ok(())
    }
}

impl<T: BlockData> VoxelStorageRegion<T> for VoxelWorld<T> {
    fn get_slice(&self, region: Region) -> VoxelWorldSlice<T> {
        let mut slice = VoxelWorldSlice::new(region);
        let region_chunks = Region::from_points(region.min() >> 4, region.max() >> 4);

        for sector in self.sectors.iter().filter(|s| {
            Region::SECTOR
                .shift(s.sector_coords << 8)
                .intersects(region)
        }) {
            for chunk_coords in Region::intersection(
                &Region::CHUNK.shift(sector.sector_coords << 4),
                &region_chunks,
            )
            .unwrap()
            .iter()
            {
                let chunk_index = Region::CHUNK.point_to_index(chunk_coords & 15).unwrap();
                let Some(chunk) = &sector.chunks[chunk_index] else {
                    continue;
                };

                for block_coords in Region::intersection(
                    &Region::CHUNK.shift(chunk.chunk_coords << 4), // <br>
                    &region,
                )
                .unwrap()
                .iter()
                {
                    let block_index = Region::CHUNK.point_to_index(block_coords & 15).unwrap();
                    let data = chunk.blocks[block_index];
                    slice.set_block(block_coords, data).unwrap();
                }
            }
        }

        slice
    }

    fn fill_slice(&mut self, slice: VoxelWorldSlice<T>) -> Result<()> {
        let region = slice.get_region();
        if !self.validate_region_loaded(region) {
            bail!("World Slice at {} overlaps unloaded chunks", region);
        }

        for chunk_coords in Region::from_points(region.min() >> 4, region.max() >> 4).iter() {
            let sector_coords = chunk_coords >> 4;
            let sector = self
                .sectors
                .iter_mut()
                .find(|s| s.sector_coords == sector_coords)
                .unwrap();

            let chunk_index = Region::CHUNK.point_to_index(chunk_coords & 15).unwrap();
            let chunk = sector.chunks[chunk_index].as_mut().unwrap();

            for block_coords in Region::intersection(
                &Region::CHUNK.shift(chunk_coords << 4), // <br>
                &region,
            )
            .unwrap()
            .iter()
            {
                let block_index = Region::CHUNK.point_to_index(block_coords & 15).unwrap();
                let data = slice.get_block(block_coords);
                chunk.blocks[block_index] = data;
            }
        }

        Ok(())
    }
}

impl<T: BlockData> ChunkStorage<VoxelWorld<T>, T> for VoxelWorld<T> {
    fn prepare_chunk(&mut self, chunk_coords: IVec3) -> Result<()> {
        let sector_coords = chunk_coords >> 4;
        let sector = match self
            .sectors
            .iter_mut()
            .find(|s| s.sector_coords == sector_coords)
        {
            Some(s) => s,
            None => {
                self.sectors.push(VoxelSector::new(sector_coords));
                self.sectors.last_mut().unwrap()
            },
        };

        let chunk_index = Region::CHUNK.point_to_index(chunk_coords & 15).unwrap();
        if sector.chunks[chunk_index].is_some() {
            bail!("Chunk ({}) already exists", chunk_coords);
        }

        sector.chunks[chunk_index] =
            Some(VoxelChunk::<T>::new(chunk_coords, ChunkLoadState::Loading));

        Ok(())
    }

    fn init_chunk(&mut self, chunk_coords: IVec3) -> InitChunkResult<VoxelWorld<T>, T> {
        let sector_coords = chunk_coords >> 4;
        let sector = match self
            .sectors
            .iter_mut()
            .find(|s| s.sector_coords == sector_coords)
        {
            Some(s) => s,
            None => {
                self.sectors.push(VoxelSector::new(sector_coords));
                self.sectors.last_mut().unwrap()
            },
        };

        let chunk_index = Region::CHUNK.point_to_index(chunk_coords & 15).unwrap();
        if let Some(mut chunk) = sector.chunks[chunk_index].as_mut() {
            if chunk.load_state == ChunkLoadState::Loaded {
                return InitChunkResult::new(
                    self,
                    Err(anyhow!("Chunk ({}) already exists", chunk_coords)),
                    chunk_coords,
                );
            }

            chunk.load_state = ChunkLoadState::Loaded;
        } else {
            sector.chunks[chunk_index] =
                Some(VoxelChunk::<T>::new(chunk_coords, ChunkLoadState::Loaded));
        }

        InitChunkResult::new(self, Ok(()), chunk_coords)
    }

    fn unload_chunk(&mut self, chunk_coords: IVec3) -> UnloadChunkResult {
        let sector_coords = chunk_coords >> 4;
        let Some(sector) = self.sectors.iter_mut().find(|s| s.sector_coords == sector_coords) else {
            return UnloadChunkResult(Err(anyhow!("Chunk ({}) does not exist", chunk_coords)));
        };

        let chunk_index = Region::CHUNK.point_to_index(chunk_coords & 15).unwrap();
        let Some(_) = sector.chunks[chunk_index] else {
            return UnloadChunkResult(Err(anyhow!("Chunk ({}) does not exist", chunk_coords)));
        };

        sector.chunks[chunk_index] = None;
        UnloadChunkResult(Ok(chunk_coords))
    }

    fn unload_all_chunks(&mut self) -> UnloadAllChunksResult {
        let mut chunk_list = vec![];

        for sector in self.sectors.iter() {
            for local_chunk_coords in Region::CHUNK.iter() {
                let chunk_index = Region::CHUNK.point_to_index(local_chunk_coords).unwrap();
                if sector.chunks[chunk_index].is_some() {
                    let chunk_coords = local_chunk_coords + (sector.sector_coords << 4);
                    chunk_list.push(chunk_coords);
                }
            }
        }

        self.sectors.clear();
        UnloadAllChunksResult(chunk_list)
    }

    fn get_chunk_load_state(&self, chunk_coords: IVec3) -> ChunkLoadState {
        let sector_coords = chunk_coords >> 4;
        let Some(sector) = self.sectors.iter().find(|s| s.sector_coords == sector_coords) else {
            return ChunkLoadState::Unloaded;
        };

        let chunk_index = Region::CHUNK.point_to_index(chunk_coords & 15).unwrap();
        let Some(chunk) = &sector.chunks[chunk_index] else {
            return ChunkLoadState::Unloaded;
        };

        chunk.load_state
    }
}

impl<T: BlockData> VoxelWorld<T> {
    /// Checks to ensure that all chunks within the provided region are loaded.
    ///
    /// This function returns true if all chunks are loaded and accessible. This
    /// function returns false, otherwise.
    pub fn validate_region_loaded(&self, region: Region) -> bool {
        let region = Region::from_points(region.min() >> 4, region.max() >> 4);
        for chunk_coords in region.iter() {
            let sector_coords = chunk_coords >> 4;
            let Some(sector) = self.sectors.iter().find(|s| s.sector_coords == sector_coords) else {
                return false;
            };

            let chunk_index = Region::CHUNK.point_to_index(chunk_coords & 15).unwrap();
            if sector.chunks[chunk_index].is_none() {
                return false;
            }
        }

        true
    }
}
