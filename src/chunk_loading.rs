use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    chunk::CHUNK_SIZE,
    positions::{index_to_chunk_pos_bounds, ChunkPos},
};

pub struct ChunkLoaderPlugin;

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreUpdate, ChunkLoader::detect_move);
    }
}

#[derive(Component)]
pub struct ChunkLoader {
    pub prev_chunk_pos: ChunkPos,

    // Chunks to check in a frame
    pub chunks_per_frame: usize,

    // Offset grid sampling across frames
    pub data_offset: usize,
    pub mesh_offset: usize,

    // Loading queues for chunk data and meshes
    pub data_load_queue: Vec<ChunkPos>,
    pub mesh_load_queue: Vec<ChunkPos>,

    // Unloading queues for chunk data and meshes
    pub data_unload_queue: VecDeque<ChunkPos>,
    pub mesh_unload_queue: VecDeque<ChunkPos>,

    // When the loader is moved, these offsets identify which chunks need to be checked
    pub data_sampling_offsets: Vec<ChunkPos>,
    pub mesh_sampling_offsets: Vec<ChunkPos>,
}

impl ChunkLoader {
    pub fn new(load_distance: u32) -> Self {
        let data_distance = load_distance + 1;
        let mesh_distance = load_distance;

        let data_sampling_offsets = Self::make_spherical_offsets(data_distance);
        let mesh_sampling_offsets = Self::make_spherical_offsets(mesh_distance);

        Self {
            chunks_per_frame: CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE,
            prev_chunk_pos: ChunkPos::new(999, 999, 999),
            data_offset: 0,
            mesh_offset: 0,
            data_load_queue: Vec::new(),
            mesh_load_queue: Vec::new(),
            data_unload_queue: VecDeque::new(),
            mesh_unload_queue: VecDeque::new(),
            data_sampling_offsets,
            mesh_sampling_offsets,
        }
    }

    fn make_spherical_offsets(radius: u32) -> Vec<ChunkPos> {
        let r = (radius * 2) + 1;

        let mut sampling_offsets = Vec::new();
        for i in 0..r * r * r {
            let mut chunk_pos = index_to_chunk_pos_bounds(i as usize, r);
            chunk_pos -= ChunkPos::splat(r as i32 / 2);

            sampling_offsets.push(chunk_pos);
        }

        // Sort offsets by the distance from origin
        let origin = (0, 0, 0).into();
        sampling_offsets.sort_by(|lhs, rhs| {
            lhs.distance_squared(origin)
                .cmp(&rhs.distance_squared(origin))
        });

        sampling_offsets
    }

    fn detect_move(mut loaders: Query<(&mut ChunkLoader, &GlobalTransform)>) {
        todo!()
    }
}
