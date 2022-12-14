//! A voxel sector component for grouping together chunks in a data structure
//! that is faster to query.

use bevy::prelude::*;

use crate::prelude::Region;

/// The depth value of the cache, to determine the memory size of one block.
const CACHE_DEPTH: u8 = 5;

/// The total number of chunk pointers within a in cache block.
const CACHE_SIZE: usize = usize::pow(1 << CACHE_DEPTH as usize, 3);

/// The cached entity pointers for chunks based on their coordinates.
struct Sector {
    /// The chunk entity pointers stored within this sector.
    chunks: Box<[Option<Entity>; CACHE_SIZE]>,

    /// The coordinates of this sector.
    sector_coords: IVec3,

    /// The total number of active chunk pointers within this sector.
    active_chunks: usize,
}

impl Sector {
    /// Creates a new sector with the given sector coordinates.
    fn new(sector_coords: IVec3) -> Self {
        Self {
            sector_coords,
            chunks: Box::new([None; CACHE_SIZE]),
            active_chunks: 0,
        }
    }

    /// Gets the region that this sector occupies.
    fn region(&self) -> Region {
        Region::from_size(self.sector_coords << CACHE_DEPTH, IVec3::ONE << CACHE_DEPTH).unwrap()
    }

    /// Gets the entity of the chunk at the given chunk coordinates.
    fn get_chunk_entity(&self, chunk_coords: IVec3) -> Option<Entity> {
        let index = self.region().point_to_index(chunk_coords).unwrap();
        self.chunks[index]
    }

    /// Sets the entity pointer at the given chunk coordinates.
    fn set_chunk_entity(&mut self, chunk_coords: IVec3, entity: Option<Entity>) {
        let index = self.region().point_to_index(chunk_coords).unwrap();

        match (self.chunks[index], entity) {
            (Some(_), None) => self.active_chunks -= 1,
            (None, Some(_)) => self.active_chunks += 1,
            _ => {},
        }

        self.chunks[index] = entity;
    }

    /// Checks if this sector is currently empty.
    fn is_empty(&self) -> bool {
        self.active_chunks == 0
    }
}

/// This component is used to quickly find entity ids of chunks based on their
/// chunk coordinates.
///
/// This component works by caching the entity ids of chunks, and must be
/// updated each time a new chunk entity is spawned or despawned.
#[derive(Component, Reflect, Default)]
pub struct ChunkEntityPointers {
    /// A list of sectors that are currently active.
    #[reflect(ignore)]
    sectors: Vec<Sector>,
}

impl ChunkEntityPointers {
    /// Gets the entity id of the chunk at the given chunk coordinates.
    ///
    /// If there is no known chunk at the given coordinates, then None is
    /// returned.
    pub fn get_chunk_entity(&self, chunk_coords: IVec3) -> Option<Entity> {
        let sector_coords = chunk_coords >> CACHE_DEPTH;
        self.sectors
            .iter()
            .find(|c| c.sector_coords == sector_coords)?
            .get_chunk_entity(chunk_coords)
    }

    /// Sets the entity id of the chunk at the given coordinates.
    pub fn set_chunk_entity(&mut self, chunk_coords: IVec3, entity: Option<Entity>) {
        let sector_coords = chunk_coords >> CACHE_DEPTH;
        let sector = match self
            .sectors
            .iter_mut()
            .find(|c| c.sector_coords == sector_coords)
        {
            Some(s) => s,
            None => {
                let sector = Sector::new(sector_coords);
                self.sectors.push(sector);
                self.sectors.last_mut().unwrap()
            },
        };

        sector.set_chunk_entity(chunk_coords, entity);

        if sector.is_empty() {
            let index = self
                .sectors
                .iter()
                .position(|s| s.sector_coords == sector_coords)
                .unwrap();
            self.sectors.remove(index);
        }
    }
}
