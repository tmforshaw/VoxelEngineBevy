use std::collections::VecDeque;

use bevy::prelude::*;

use crate::positions::ChunkPos;

pub struct ChunkLoaderPlugin;

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreUpdate, ());
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

        // let data_sampling_offsets = m

        // Self {}

        todo!()
    }

    fn make_spherical_offsets(radius: u32) -> Vec<ChunkPos> {}
}
