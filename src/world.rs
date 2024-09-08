use std::{
    collections::HashMap,
    ops::Index,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;

use crate::{
    chunk::{Chunk, CHUNK_SIZE},
    chunk_mesh,
    positions::{ChunkPos, WorldPos},
    voxel::{Voxel, VoxelPos, VoxelType},
};

#[derive(Resource, Default)]
pub struct World {
    pub chunks: HashMap<ChunkPos, Arc<Mutex<Chunk>>>,
}

impl World {
    pub fn new_with(voxels_at: Vec<WorldPos>) -> Self {
        let mut world = World::default();

        // world.add_chunk_at((0, -1, 0).into());
        // world.add_chunk_at((0, -2, 0).into());
        // world.add_chunk_at((1, -1, 0).into());
        // world.add_chunk_at((1, -2, 0).into());
        // world.add_chunk_at((2, -1, 0).into());
        // world.add_chunk_at((2, -2, 0).into());
        // world.add_chunk_at((3, -1, 0).into());
        // world.add_chunk_at((3, -2, 0).into());

        for pos in voxels_at {
            world.set_voxel(pos, VoxelType::Block);
        }

        world
    }

    pub fn generate(
        world: Res<World>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for (chunk_pos, chunk) in world.chunks.iter() {
            for voxel_index in 0..(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) {
                let (x, y, z): (usize, usize, usize) = VoxelPos::from_index(voxel_index).into();

                if let Ok(chunk) = chunk.lock() {
                    if chunk[voxel_index].voxel_type != VoxelType::Air {
                        let mesh_handle = meshes.add(chunk_mesh::generate_mesh());

                        let hue = ((x * CHUNK_SIZE + y) * CHUNK_SIZE + z) as f32
                            * (360. / (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as f32);

                        commands.spawn(PbrBundle {
                            mesh: mesh_handle,
                            material: materials.add(Color::hsv(hue, 1., 1.)),
                            transform: Transform::from_xyz(
                                (x as i32 + chunk_pos.x * CHUNK_SIZE as i32) as f32,
                                (y as i32 + chunk_pos.y * CHUNK_SIZE as i32) as f32,
                                (z as i32 + chunk_pos.z * CHUNK_SIZE as i32) as f32,
                            ),
                            ..default()
                        });
                    }
                }
            }
        }
    }

    pub fn add_chunk_at(&mut self, chunk_pos: ChunkPos) {
        self.chunks.insert(
            chunk_pos,
            Arc::from(Mutex::from(Chunk::new_from_noise(chunk_pos))),
        );
    }

    pub fn set_voxels_in_chunk(&mut self, chunk_pos: ChunkPos, voxels: Vec<(VoxelPos, VoxelType)>) {
        // If the chunk doesn't exist, it creates it in the hashmap
        // Then modifies it to have the specified voxels
        self.chunks
            .entry(chunk_pos)
            .and_modify(|chunk| {
                if let Ok(mut chunk) = chunk.lock() {
                    chunk.set_voxels(voxels.clone());
                }
            })
            .or_insert(Arc::from(Mutex::from(Chunk::with_voxels(voxels))));
    }

    pub fn set_voxel(&mut self, voxel_pos: WorldPos, voxel_type: VoxelType) {
        self.set_voxels(vec![(voxel_pos, voxel_type)]);
    }

    pub fn set_voxels(&mut self, voxels: Vec<(WorldPos, VoxelType)>) {
        // let (chunk_pos, voxel_pos) = WorldPos::world_to_voxel_pos(pos);

        let voxels_with_chunk = voxels
            .iter()
            .map(|&(world_pos, voxel_type)| {
                let (voxel_pos, chunk_pos) = WorldPos::to_voxel_pos(world_pos);

                (chunk_pos, voxel_pos, voxel_type)
            })
            .fold(
                HashMap::<ChunkPos, Vec<(VoxelPos, VoxelType)>>::new(),
                |mut acc, (chunk_pos, voxel_pos, voxel_type)| {
                    acc.entry(chunk_pos)
                        .and_modify(|vec| vec.push((voxel_pos, voxel_type)))
                        .or_insert(vec![(voxel_pos, voxel_type)]);

                    acc
                },
            );

        for (chunk_pos, voxels) in voxels_with_chunk {
            self.set_voxels_in_chunk(chunk_pos, voxels);
        }
    }

    // Used to get a voxel (including from neighbouring chunks)
    pub fn get_voxel(&self, world_pos: WorldPos) -> Option<Voxel> {
        // let (chunk_x, x) = if pos.x > 0 {
        //     (0, CHUNK_SIZE as i32 + pos.x)
        // } else if pos.x >= CHUNK_SIZE as i32 {
        //     (2, pos.x - CHUNK_SIZE as i32)
        // } else {
        //     (1, pos.x)
        // };

        // let (chunk_y, y) = if pos.y > 0 {
        //     (0, CHUNK_SIZE as i32 + pos.y)
        // } else if pos.y >= CHUNK_SIZE as i32 {
        //     (2, pos.y - CHUNK_SIZE as i32)
        // } else {
        //     (1, pos.y)
        // };

        // let (chunk_z, z) = if pos.z > 0 {
        //     (0, CHUNK_SIZE as i32 + pos.z)
        // } else if pos.z >= CHUNK_SIZE as i32 {
        //     (2, pos.z - CHUNK_SIZE as i32)
        // } else {
        //     (1, pos.z)
        // };

        let (voxel_pos, chunk_pos) = WorldPos::to_voxel_pos(world_pos);

        // let chunk_pos = (
        //     chunk_pos.x + chunk_x,
        //     chunk_pos.y + chunk_y,
        //     chunk_pos.z + chunk_z,
        // )
        //     .into();

        // If the chunk exists
        if let Some(chunk) = self.chunks.get(&chunk_pos) {
            // Lock the mutex
            if let Ok(chunk) = chunk.lock() {
                Some(chunk[voxel_pos])
            } else {
                None
            }
        } else {
            Some(Voxel::new(VoxelType::Air))
        }
    }

    // Returns current, back, left, down
    pub fn get_adjacent_voxels(
        &self,
        voxel_pos: VoxelPos,
        chunk_pos: ChunkPos,
    ) -> (Option<Voxel>, Option<Voxel>, Option<Voxel>, Option<Voxel>) {
        let world_pos = WorldPos::from_voxel_pos(voxel_pos, chunk_pos);

        // println!("{voxel_pos:?}, {chunk_pos:?}, {world_pos:?}");

        let current = self.get_voxel(world_pos);
        let back = self.get_voxel((world_pos.x, world_pos.y, world_pos.z - 1).into());
        let left = self.get_voxel((world_pos.x - 1, world_pos.y, world_pos.z).into());
        let down = self.get_voxel((world_pos.x, world_pos.y - 1, world_pos.z).into());

        (current, back, left, down)
    }
}
