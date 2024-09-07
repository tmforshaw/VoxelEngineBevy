// World Position Struct (For any voxel in the world)

use crate::{chunk::CHUNK_SIZE, voxel::VoxelPos};

#[derive(Copy, Clone)]
pub struct WorldPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn to_voxel_pos(pos: Self) -> (ChunkPos, VoxelPos) {
        // Subtract CHUNK_SIZE / 2 before modulus so that negative chunks are rounded down to negative values (instead of rounded up to 0,0,0)
        // Add 0.5 before division so that before rounding, a value of 1/(2 * CHUNK_SIZE) is added, this makes the even rounding work for any chunk size
        let chunk_pos = (
            (((pos.x - CHUNK_SIZE as i32 / 2) as f32 + 0.5) / CHUNK_SIZE as f32).round_ties_even(),
            (((pos.y - CHUNK_SIZE as i32 / 2) as f32 + 0.5) / CHUNK_SIZE as f32).round_ties_even(),
            (((pos.z - CHUNK_SIZE as i32 / 2) as f32 + 0.5) / CHUNK_SIZE as f32).round_ties_even(),
        );

        // Have to add CHUNK_SIZE after the modulus to make it a true modulus function instead of just remainder (which includes negatives)
        let voxel_pos = (
            ((pos.x % CHUNK_SIZE as i32 + CHUNK_SIZE as i32) % CHUNK_SIZE as i32) as usize,
            ((pos.y % CHUNK_SIZE as i32 + CHUNK_SIZE as i32) % CHUNK_SIZE as i32) as usize,
            ((pos.z % CHUNK_SIZE as i32 + CHUNK_SIZE as i32) % CHUNK_SIZE as i32) as usize,
        );

        println!("{chunk_pos:?}\t{voxel_pos:?}");
        let chunk_pos = (chunk_pos.0 as i32, chunk_pos.1 as i32, chunk_pos.2 as i32);

        (chunk_pos.into(), voxel_pos.into())
    }
}

impl From<(i32, i32, i32)> for WorldPos {
    fn from(pos: (i32, i32, i32)) -> Self {
        Self {
            x: pos.0,
            y: pos.1,
            z: pos.2,
        }
    }
}

// Chunk Position Struct (For the position of a chunk in the world)

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_tuple(pos: (i32, i32, i32)) -> Self {
        Self {
            x: pos.0,
            y: pos.1,
            z: pos.2,
        }
    }

    pub fn to_tuple(self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }
}

impl From<(i32, i32, i32)> for ChunkPos {
    fn from(pos: (i32, i32, i32)) -> Self {
        Self::from_tuple(pos)
    }
}

impl From<ChunkPos> for (i32, i32, i32) {
    fn from(chunk_pos: ChunkPos) -> Self {
        chunk_pos.to_tuple()
    }
}

// Voxel Position Struct (For the position of a voxel within a chunk)
