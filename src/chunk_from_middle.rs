use std::{collections::HashMap, sync::Arc};

use bevy::math::IVec3;

use crate::{
    chunk::Chunk,
    constants::{CHUNKS_FROM_MIDDLE_SIZE, CHUNK_SIZE},
    positions::{chunk_pos_to_index_bounds, index_to_chunk_pos_bounds, ChunkPos, VoxelPos},
    voxel::Voxel,
};

// pointers to chunk data, a middle one with all their neighbours
#[derive(Clone)]
pub struct ChunksFromMiddle {
    pub chunks: Vec<Arc<Chunk>>,
}

impl ChunksFromMiddle {
    // Construct a ChunksFromMiddle around a central chunk
    pub fn try_new(
        chunk_hashmap: &HashMap<ChunkPos, Arc<Chunk>>,
        middle_chunk: ChunkPos,
    ) -> Option<Self> {
        let mut chunks = Vec::new();

        for index in 0..CHUNKS_FROM_MIDDLE_SIZE * CHUNKS_FROM_MIDDLE_SIZE * CHUNKS_FROM_MIDDLE_SIZE
        {
            let offset = index_to_chunk_pos_bounds(index, CHUNKS_FROM_MIDDLE_SIZE as u32)
                + ChunkPos::splat(-1);
            chunks.push(Arc::clone(
                chunk_hashmap.get(&(middle_chunk + offset)).unwrap(),
            ));
        }

        Some(Self { chunks })
    }

    pub fn get_voxel(&self, voxel_pos_ivec3: IVec3) -> &Voxel {
        let voxel_pos = VoxelPos::from_ivec3(voxel_pos_ivec3 + IVec3::splat(CHUNK_SIZE as i32));
        let chunk_pos = (voxel_pos / CHUNK_SIZE).to_i32().into();

        // Take modulus of x, y, and z with respect to CHUNK_SIZE, adding CHUNK_SIZE so that negative values don't appear
        let voxel_pos = voxel_pos % CHUNK_SIZE;
        let chunk_index = chunk_pos_to_index_bounds(chunk_pos, CHUNKS_FROM_MIDDLE_SIZE as u32);

        &(&self.chunks[chunk_index])[voxel_pos]
    }

    pub fn get_voxel_no_neighbour(&self, voxel_pos: VoxelPos) -> &Voxel {
        //  TODO i dont know why 13 is the middle chunk
        &(&self.chunks[13])[voxel_pos]
    }

    // Returns current, back, left, down
    pub fn get_adjacent_voxels(
        &self,
        voxel_pos: VoxelPos,
        // chunk_pos: ChunkPos,
    ) -> (&Voxel, &Voxel, &Voxel, &Voxel) {
        // let world_pos = WorldPos::from_voxel_pos(voxel_pos, chunk_pos);

        let pos_ivec3 = voxel_pos.to_ivec3();

        let current = self.get_voxel(pos_ivec3); // Should always be able to find current voxel
        let back = self.get_voxel((pos_ivec3.x, pos_ivec3.y, pos_ivec3.z - 1).into());
        let left = self.get_voxel((pos_ivec3.x - 1, pos_ivec3.y, pos_ivec3.z).into());
        let down = self.get_voxel((pos_ivec3.x, pos_ivec3.y - 1, pos_ivec3.z).into());

        (current, back, left, down)
    }

    pub fn are_all_voxels_same(&self) -> bool {
        // If there is only one voxel, keep checking, otherwise return false
        if self.chunks[0].len() == 1 {
            let block = self.chunks[0][0];
            for chunk in self.chunks[1..].iter() {
                if chunk.len() == 1 {
                    // If the first block of each chunk is different to the first chunk's then return false
                    if block.voxel_type != chunk[0].voxel_type {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        } else {
            return false;
        }

        true
    }
}
