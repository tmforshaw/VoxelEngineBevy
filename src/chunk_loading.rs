use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashSet};

use crate::{
    constants::{ADJACENT_CHUNK_DIRECTIONS, CHUNK_SIZE, MAX_CHUNK_LOADS, MAX_DATA_TASKS},
    positions::{index_to_chunk_pos_bounds, ChunkPos},
    world::World,
};

pub struct ChunkLoaderPlugin;

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            PreUpdate,
            (
                ChunkLoader::detect_move,
                ChunkLoader::load_chunks,
                ChunkLoader::unload_chunks,
                ChunkLoader::load_mesh,
                ChunkLoader::unload_mesh,
            ),
        );
    }
}

#[derive(Component, Debug)]
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

    fn detect_move(
        mut loaders: Query<(&mut ChunkLoader, &GlobalTransform)>,
        mut world: ResMut<World>,
    ) {
        for (mut loader, g_transform) in loaders.iter_mut() {
            let chunk_pos = ChunkPos::from_vec3(
                (g_transform.translation() - Vec3::splat(CHUNK_SIZE as f32 / 2.))
                    / CHUNK_SIZE as f32,
            );

            let prev_chunk_pos = loader.prev_chunk_pos;
            let chunk_pos_has_changed = chunk_pos != prev_chunk_pos;
            if !chunk_pos_has_changed {
                return;
            }
            loader.prev_chunk_pos = chunk_pos;

            let load_data_area = loader
                .data_sampling_offsets
                .iter()
                .map(|offset| chunk_pos + *offset)
                .collect::<HashSet<ChunkPos>>();

            let unload_data_area = loader
                .data_sampling_offsets
                .iter()
                .map(|offset| prev_chunk_pos + *offset)
                .collect::<HashSet<ChunkPos>>();

            let load_mesh_area = loader
                .mesh_sampling_offsets
                .iter()
                .map(|offset| chunk_pos + *offset)
                .collect::<HashSet<ChunkPos>>();

            let unload_mesh_area = loader
                .mesh_sampling_offsets
                .iter()
                .map(|offset| prev_chunk_pos + *offset)
                .collect::<HashSet<ChunkPos>>();

            let data_load = load_data_area.difference(&unload_data_area);
            let data_unload = unload_data_area.difference(&load_data_area);
            let mesh_load = load_mesh_area.difference(&unload_mesh_area);
            let mesh_unload = unload_mesh_area.difference(&load_mesh_area);

            loader.data_load_queue.extend(data_load);
            loader.data_unload_queue.extend(data_unload);
            loader.mesh_load_queue.extend(mesh_load);
            loader.mesh_unload_queue.extend(mesh_unload);

            let ChunkLoader {
                data_load_queue,
                mesh_load_queue,
                data_unload_queue,
                mesh_unload_queue,
                ..
            } = loader.as_mut();

            // Remove resolved chunk data from queue
            for pos in data_unload_queue.iter() {
                if let Some((i, _)) = world
                    .load_data_queue
                    .iter()
                    .enumerate()
                    .find(|(_i, world_chunk_pos)| *world_chunk_pos == pos)
                {
                    world.load_data_queue.remove(i);
                }
            }

            // Remove resolved meshes from queue
            for pos in mesh_unload_queue.iter() {
                if let Some((i, _)) = world
                    .load_mesh_queue
                    .iter()
                    .enumerate()
                    .find(|(_i, world_chunk_pos)| *world_chunk_pos == pos)
                {
                    world.load_mesh_queue.remove(i);
                }
            }

            // Remove the unloads from load
            data_load_queue.retain(|pos| !data_unload_queue.contains(pos));
            mesh_load_queue.retain(|pos| !mesh_unload_queue.contains(pos));

            // Sort data and mesh load queues by distance to chunk_pos
            loader.data_load_queue.sort_by(|lhs, rhs| {
                lhs.distance_squared(chunk_pos)
                    .cmp(&rhs.distance_squared(chunk_pos))
            });
            loader.mesh_load_queue.sort_by(|lhs, rhs| {
                lhs.distance_squared(chunk_pos)
                    .cmp(&rhs.distance_squared(chunk_pos))
            });
        }
    }

    pub fn load_chunks(
        mut loaders: Query<(&mut ChunkLoader, &GlobalTransform)>,
        mut world: ResMut<World>,
    ) {
        for (mut loader, _g_transform) in loaders.iter_mut() {
            if world.data_tasks.len() >= MAX_DATA_TASKS {
                return;
            }

            let data_len = loader.data_load_queue.len();

            for chunk_pos in loader
                .data_load_queue
                .drain(0..MAX_CHUNK_LOADS.min(data_len))
            {
                let is_busy = world.chunks.contains_key(&chunk_pos)
                    || world.load_data_queue.contains(&chunk_pos)
                    || world.data_tasks.contains_key(&chunk_pos);

                if !is_busy {
                    world.load_data_queue.push(chunk_pos);

                    // Abort load
                    let index_of_unloading = world
                        .unload_data_queue
                        .iter()
                        .enumerate()
                        .find_map(|(i, pos)| if pos == &chunk_pos { Some(i) } else { None });

                    if let Some(i) = index_of_unloading {
                        world.unload_data_queue.remove(i);
                    }
                }
            }
        }
    }

    pub fn unload_chunks(
        mut loaders: Query<(&mut ChunkLoader, &GlobalTransform)>,
        mut world: ResMut<World>,
    ) {
        // Find all loaded and check if in range
        for (mut loader, _g_transform) in loaders.iter_mut() {
            for chunk_pos in loader.data_unload_queue.drain(..) {
                let is_busy = !world.chunks.contains_key(&chunk_pos);

                if !is_busy {
                    world.unload_data_queue.push(chunk_pos);
                }
            }
        }
    }

    pub fn load_mesh(mut loaders: Query<&mut ChunkLoader>, mut world: ResMut<World>) {
        for mut loader in loaders.iter_mut() {
            let mut retries = Vec::new();

            let mesh_data_len = loader.mesh_load_queue.len();

            for chunk_pos in loader
                .mesh_load_queue
                .drain(0..MAX_CHUNK_LOADS.min(mesh_data_len))
            {
                let mut is_busy = world.load_mesh_queue.contains(&chunk_pos);

                is_busy |= !ADJACENT_CHUNK_DIRECTIONS
                    .iter()
                    .map(|&offset| chunk_pos + offset)
                    .all(|pos| world.chunks.contains_key(&pos));

                if !is_busy {
                    world.load_mesh_queue.push(chunk_pos);

                    // Abort load
                    let index_of_unloading = world
                        .unload_mesh_queue
                        .iter()
                        .enumerate()
                        .find_map(|(i, pos)| if pos == &chunk_pos { Some(i) } else { None });

                    if let Some(i) = index_of_unloading {
                        world.unload_mesh_queue.remove(i);
                    }
                } else {
                    retries.push(chunk_pos);
                }
            }

            loader.mesh_load_queue.append(&mut retries);
        }
    }

    pub fn unload_mesh(mut loaders: Query<&mut ChunkLoader>, mut world: ResMut<World>) {
        // Find all loaded and check if in range
        for mut loader in loaders.iter_mut() {
            for chunk_pos in loader.mesh_unload_queue.drain(..) {
                world.unload_mesh_queue.push(chunk_pos);
            }
        }
    }
}
