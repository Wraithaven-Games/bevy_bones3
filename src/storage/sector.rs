//! Represents a 16x16x16 grid of chunks.

use anyhow::{anyhow, bail, Result};
use bevy::prelude::*;

use super::chunk::VoxelChunk;
use super::voxel::{
    ChunkStorage,
    InitChunkResult,
    UnloadAllChunksResult,
    UnloadChunkResult,
    VoxelStorage,
};
use super::BlockData;
use crate::math::Region;

/// A single 16x16x16 grid of chunks within a voxel world that store a single,
/// specific type of data. These chunks may optionally be defined.
#[derive(Debug)]
pub struct VoxelSector<T: BlockData> {
    /// The chunk array grid for this sector.
    chunks: Box<[Option<VoxelChunk<T>>; 4096]>,

    /// The coordinates of this sector.
    sector_coords: IVec3,
}

impl<T: BlockData> VoxelStorage<T> for VoxelSector<T> {
    fn get_block(&self, block_coords: IVec3) -> Result<T> {
        let local_block_coords = block_coords - (self.sector_coords << 8);
        let local_chunk_coords = local_block_coords >> 4;

        if let Ok(index) = Region::CHUNK.point_to_index(local_chunk_coords) {
            if let Some(chunk) = &self.chunks[index] {
                Ok(chunk.get_block(local_block_coords & 15)?)
            } else {
                Ok(T::default())
            }
        } else {
            bail!(
                "Block coordinates ({}) are outside of sector bounds ({})",
                block_coords,
                Region::from_size(self.sector_coords << 8, IVec3::new(256, 256, 256)).unwrap()
            );
        }
    }

    fn set_block(&mut self, block_coords: IVec3, data: T) -> Result<()> {
        let local_block_coords = block_coords - (self.sector_coords << 8);
        let local_chunk_coords = local_block_coords >> 4;

        if let Ok(index) = Region::CHUNK.point_to_index(local_chunk_coords) {
            if let Some(chunk) = &mut self.chunks[index] {
                chunk.set_block(local_block_coords & 15, data)?;
                Ok(())
            } else {
                bail!(
                    "Chunk ({}) has not been initialized and cannot be written to",
                    block_coords >> 4,
                );
            }
        } else {
            bail!(
                "Block coordinates ({}) are outside of sector bounds ({})",
                block_coords,
                Region::from_size(self.sector_coords << 8, IVec3::new(256, 256, 256)).unwrap()
            );
        }
    }
}

impl<T: BlockData> ChunkStorage for VoxelSector<T> {
    fn init_chunk(&mut self, chunk_coords: IVec3) -> InitChunkResult {
        let local_chunk_coords = chunk_coords - (self.sector_coords << 4);

        if let Ok(index) = Region::CHUNK.point_to_index(local_chunk_coords) {
            if self.chunks[index].is_some() {
                InitChunkResult(Err(anyhow!("Chunk ({}) already exists", chunk_coords)))
            } else {
                self.chunks[index] = Some(VoxelChunk::<T>::new(chunk_coords));
                InitChunkResult(Ok(chunk_coords))
            }
        } else {
            InitChunkResult(Err(anyhow!(
                "Chunk coordinates ({}) are outside of sector bounds ({})",
                chunk_coords,
                Region::from_size(self.sector_coords << 4, IVec3::new(16, 16, 16)).unwrap()
            )))
        }
    }

    fn unload_chunk(&mut self, chunk_coords: IVec3) -> UnloadChunkResult {
        let local_chunk_coords = chunk_coords - (self.sector_coords << 4);

        if let Ok(index) = Region::CHUNK.point_to_index(local_chunk_coords) {
            if self.chunks[index].is_some() {
                self.chunks[index] = None;
                UnloadChunkResult(Ok(chunk_coords))
            } else {
                UnloadChunkResult(Err(anyhow!("Chunk ({}) does not exist", chunk_coords)))
            }
        } else {
            UnloadChunkResult(Err(anyhow!(
                "Chunk coordinates ({}) are outside of sector bounds ({})",
                chunk_coords,
                Region::from_size(self.sector_coords << 4, IVec3::new(16, 16, 16)).unwrap()
            )))
        }
    }

    fn unload_all_chunks(&mut self) -> UnloadAllChunksResult {
        let mut unloaded = vec![];

        for local_chunk_coords in Region::CHUNK.iter() {
            let index = Region::CHUNK.point_to_index(local_chunk_coords).unwrap();
            if self.chunks[index].is_some() {
                self.chunks[index] = None;

                let chunk_coords = local_chunk_coords + (self.sector_coords << 4);
                unloaded.push(chunk_coords);
            }
        }

        UnloadAllChunksResult(unloaded)
    }

    fn is_chunk_loaded(&self, chunk_coords: IVec3) -> Result<bool> {
        let local_chunk_coords = chunk_coords - (self.sector_coords << 4);

        if let Ok(index) = Region::CHUNK.point_to_index(local_chunk_coords) {
            if self.chunks[index].is_some() {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            bail!(
                "Chunk coordinates ({}) are outside of sector bounds ({})",
                chunk_coords,
                Region::from_size(self.sector_coords << 4, IVec3::new(16, 16, 16)).unwrap()
            );
        }
    }
}

impl<T: BlockData> VoxelSector<T> {
    /// Creates a new, empty sector instance at the given coordinates.
    pub fn new(sector_coords: IVec3) -> Self {
        Self {
            chunks: Box::new([(); 4096].map(|_| None)),
            sector_coords,
        }
    }

    /// Gets the coordinate location of this voxel sector.
    pub fn get_sector_coords(&self) -> IVec3 {
        self.sector_coords
    }

    /// Gets an iterator over all loaded chunks within this sector.
    pub fn chunk_iter(&self) -> impl Iterator<Item = &VoxelChunk<T>> {
        self.chunks.iter().flatten()
    }
}
