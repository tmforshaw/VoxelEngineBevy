use bracket_noise::prelude::*;

use crate::{
    positions::{ChunkPos, VoxelPos, WorldPos},
    voxel::{Voxel, VoxelType},
};

pub const NOISE_SEED: u64 = 0;

pub const CHUNK_SIZE: usize = 32;

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_from_noise(chunk_pos: ChunkPos) -> Self {
        let mut noise = FastNoise::seeded(NOISE_SEED);
        noise.set_noise_type(NoiseType::Perlin);
        noise.set_frequency(0.025);

        let mut voxels = [Voxel::new(VoxelType::Air); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        (0..voxels.len()).for_each(|index| {
            let voxel_pos = VoxelPos::from_index(index);
            let world_pos = WorldPos::from_voxel_pos(voxel_pos, chunk_pos);

            let noise_val = noise.get_noise(world_pos.x as f32 + 0.5, world_pos.z as f32 + 0.5);
            let height = noise_val * 30.0;

            let solid = height > world_pos.y as f32;

            let voxel_type = if solid {
                VoxelType::Block
            } else {
                VoxelType::Air
            };

            voxels[index] = Voxel::new(voxel_type);
        });

        Chunk { voxels }
    }

    pub fn set_voxel(&mut self, voxel_pos: VoxelPos, voxel_type: VoxelType) {
        // Check that the position is within the chunk
        assert!(
            voxel_pos.x < CHUNK_SIZE && voxel_pos.y < CHUNK_SIZE && voxel_pos.z < CHUNK_SIZE,
            "x: {}, y: {}, z: {}",
            voxel_pos.x,
            voxel_pos.y,
            voxel_pos.z
        );

        self[voxel_pos].voxel_type = voxel_type;
    }

    pub fn set_voxels(&mut self, voxels: Vec<(VoxelPos, VoxelType)>) {
        for (voxel_pos, voxel_type) in voxels {
            self.set_voxel(voxel_pos, voxel_type);
        }
    }

    pub fn with_voxels(voxels: Vec<(VoxelPos, VoxelType)>) -> Self {
        let mut chunk = Self::default();

        for (voxel_pos, voxel_type) in voxels {
            chunk.voxels[VoxelPos::to_index(voxel_pos)].voxel_type = voxel_type;
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
