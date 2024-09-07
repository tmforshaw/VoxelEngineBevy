use crate::voxel::{Voxel, VoxelPos, VoxelType};

pub const CHUNK_SIZE: usize = 4;

#[derive(Clone, Debug)]
pub struct Chunk {
    voxels: [Voxel; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            voxels: [Voxel::default(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

impl Chunk {
    pub fn set_voxel(&mut self, voxel_pos: VoxelPos, voxel_type: VoxelType) {
        // Check that the position is within the chunk
        assert!(
            voxel_pos.x < CHUNK_SIZE && voxel_pos.y < CHUNK_SIZE && voxel_pos.z < CHUNK_SIZE,
            "x: {}, y: {}, z: {}",
            voxel_pos.x,
            voxel_pos.y,
            voxel_pos.z
        );

        self[voxel_pos].kind = voxel_type;
    }

    pub fn set_voxels(&mut self, voxels: Vec<(VoxelPos, VoxelType)>) {
        for (voxel_pos, voxel_type) in voxels {
            self.set_voxel(voxel_pos, voxel_type);
        }
    }

    pub fn with_voxels(voxels: Vec<(VoxelPos, VoxelType)>) -> Self {
        let mut chunk = Self::default();

        for (voxel_pos, voxel_type) in voxels {
            chunk.voxels[VoxelPos::to_index(voxel_pos)].kind = voxel_type;
        }

        chunk
    }
}

impl std::ops::Index<usize> for Chunk {
    type Output = Voxel;

    fn index(&self, index: usize) -> &Self::Output {
        &self.voxels[index]
    }
}

impl std::ops::IndexMut<usize> for Chunk {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.voxels[index]
    }
}

impl std::ops::Index<VoxelPos> for Chunk {
    type Output = Voxel;

    fn index(&self, index: VoxelPos) -> &Self::Output {
        &self.voxels[VoxelPos::to_index(index)]
    }
}

impl std::ops::IndexMut<VoxelPos> for Chunk {
    fn index_mut(&mut self, index: VoxelPos) -> &mut Self::Output {
        &mut self.voxels[VoxelPos::to_index(index)]
    }
}

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
