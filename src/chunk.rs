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
