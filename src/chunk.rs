use bracket_noise::prelude::*;

use crate::{
    constants::{CHUNK_SIZE, NOISE_FREQUENCY, NOISE_HEIGHT_SCALE, NOISE_SEED},
    positions::{ChunkPos, VoxelPos, WorldPos},
    voxel::{Voxel, VoxelType},
};

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
        noise.set_noise_type(NoiseType::PerlinFractal);
        noise.set_frequency(NOISE_FREQUENCY * 1.5);
        noise.set_fractal_octaves(8);
        noise.set_fractal_lacunarity(2.);
        noise.set_fractal_gain(0.25);

        let mut voxels = [Voxel::default(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        (0..voxels.len()).for_each(|index| {
            let voxel_pos = VoxelPos::from_index(index);
            let world_pos = WorldPos::from_voxel_pos(voxel_pos, chunk_pos);

            // let overhang =
            //     noise.get_noise3d(voxel_pos.x as f32, voxel_pos.y as f32, voxel_pos.z as f32)
            //         * 55.0;

            let noise_val =
                noise.get_noise3d(world_pos.x as f32, world_pos.y as f32, world_pos.z as f32);
            let height = noise_val * NOISE_HEIGHT_SCALE;

            let solid = height > world_pos.y as f32;
            // let solid = height > NOISE_HEIGHT_SCALE * 0.25;

            // let solid = world_pos.y < 10;

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
            chunk.voxels[voxel_pos.to_index()].voxel_type = voxel_type;
        }

        chunk
    }

    pub fn len(&self) -> usize {
        self.voxels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        &self.voxels[index.to_index()]
    }
}

impl std::ops::IndexMut<VoxelPos> for Chunk {
    fn index_mut(&mut self, index: VoxelPos) -> &mut Self::Output {
        &mut self.voxels[index.to_index()]
    }
}
