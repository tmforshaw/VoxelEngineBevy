use std::{collections::HashMap, sync::Arc};

use bevy::{
    prelude::*,
    render::{mesh::Indices, primitives::Aabb, render_asset::RenderAssetUsages},
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};

use crate::{
    chunk::{Chunk, CHUNK_SIZE},
    chunk_from_middle::ChunksFromMiddle,
    chunk_loading::{ChunkLoader, MAX_DATA_TASKS, MAX_MESH_TASKS},
    chunk_mesh::ChunkMesh,
    culled_mesher,
    positions::ChunkPos,
    rendering::{GlobalChunkMaterial, ATTRIBUTE_VOXEL},
};

// const NORMALS_ARRAY: [[f32; 3]; 6] = [
//     [-1.0, 0.0, 0.0], // Left
//     [1.0, 0.0, 0.0],  // Right
//     [0.0, 0.0, 1.0],  // Back
//     [0.0, 0.0, -1.0], // Front
//     [0.0, 1.0, 0.0],  // Up
//     [0.0, -1.0, 0.0], // Down
// ];

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(World::default())
            .add_systems(
                Update,
                (
                    (World::join_data, World::join_mesh),
                    (World::unload_data, World::unload_mesh),
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (World::start_data_tasks, World::start_mesh_tasks),
            );
    }
}

#[derive(Resource, Default)]
pub struct World {
    pub chunks: HashMap<ChunkPos, Arc<Chunk>>,
    pub load_data_queue: Vec<ChunkPos>,
    pub load_mesh_queue: Vec<ChunkPos>,
    pub unload_data_queue: Vec<ChunkPos>,
    pub unload_mesh_queue: Vec<ChunkPos>,
    pub data_tasks: HashMap<ChunkPos, Option<Task<Chunk>>>,
    pub mesh_tasks: Vec<(ChunkPos, Option<Task<Option<ChunkMesh>>>)>,
    pub chunk_entities: HashMap<ChunkPos, Entity>,
}

impl World {
    // Start data building tasks for the chunks in range
    pub fn start_data_tasks(
        mut world: ResMut<World>,
        loaders: Query<&GlobalTransform, With<ChunkLoader>>,
    ) {
        let task_pool = AsyncComputeTaskPool::get();

        let World {
            load_data_queue,
            data_tasks,
            ..
        } = world.as_mut();

        let g_loader = loaders.single();
        let loader_pos =
            ChunkPos::from_vec3(g_loader.translation() - Vec3::splat(CHUNK_SIZE as f32 / 2.)) / 32;

        load_data_queue.sort_by(|lhs, rhs| {
            lhs.distance_squared(loader_pos)
                .cmp(&rhs.distance_squared(loader_pos))
        });

        let tasks_left = (MAX_DATA_TASKS as i32 - data_tasks.len() as i32)
            .min(load_data_queue.len() as i32)
            .max(0) as usize;

        for chunk_pos in load_data_queue.drain(0..tasks_left) {
            let task = task_pool.spawn(async move { Chunk::new_from_noise(chunk_pos) });

            data_tasks.insert(chunk_pos, Some(task));
        }
    }

    // Destroy chunk data
    pub fn unload_data(mut world: ResMut<World>) {
        let World {
            unload_data_queue,
            chunks,
            ..
        } = world.as_mut();

        for chunk_pos in unload_data_queue.drain(..) {
            chunks.remove(&chunk_pos);
        }
    }

    pub fn start_mesh_tasks(
        mut world: ResMut<World>,
        loaders: Query<&GlobalTransform, With<ChunkLoader>>,
    ) {
        let task_pool = AsyncComputeTaskPool::get();

        let World {
            chunks,
            load_mesh_queue,
            mesh_tasks,
            ..
        } = world.as_mut();

        let loader_g = loaders.single();
        let loader_pos =
            ChunkPos::from_vec3(loader_g.translation() - Vec3::splat(CHUNK_SIZE as f32 / 2.)) / 32;

        load_mesh_queue.sort_by(|lhs, rhs| {
            lhs.distance_squared(loader_pos)
                .cmp(&rhs.distance_squared(loader_pos))
        });

        let tasks_left = (MAX_MESH_TASKS as i32 - mesh_tasks.len() as i32)
            .min(load_mesh_queue.len() as i32)
            .max(0) as usize;
        for chunk_pos in load_mesh_queue.drain(0..tasks_left) {
            let Some(chunks_from_middle) = ChunksFromMiddle::try_new(chunks, chunk_pos) else {
                continue;
            };

            let task = task_pool
                .spawn(async move { culled_mesher::build_chunk_mesh(&chunks_from_middle) });

            mesh_tasks.push((chunk_pos, Some(task)));
        }
    }

    // Destroy queued chunk mesh entities
    pub fn unload_mesh(mut commands: Commands, mut world: ResMut<World>) {
        let World {
            unload_mesh_queue,
            chunk_entities,
            ..
        } = world.as_mut();

        let mut retry = Vec::new();

        for chunk_pos in unload_mesh_queue.drain(..) {
            let Some(chunk_id) = chunk_entities.remove(&chunk_pos) else {
                continue;
            };
            if let Some(mut entity_commands) = commands.get_entity(chunk_id) {
                entity_commands.despawn();
            };
        }

        unload_mesh_queue.append(&mut retry);
    }

    // Join the chunk threads
    pub fn join_data(mut world: ResMut<World>) {
        let World {
            chunks, data_tasks, ..
        } = world.as_mut();

        for (chunk_pos, task_option) in data_tasks.iter_mut() {
            let Some(mut task) = task_option.take() else {
                warn!("Someone modified a task");
                continue;
            };

            let Some(chunk) = block_on(future::poll_once(&mut task)) else {
                // Failed to poll, keep task alive
                *task_option = Some(task);
                continue;
            };

            chunks.insert(*chunk_pos, Arc::new(chunk));
        }

        data_tasks.retain(|_chunk_pos, task_option| task_option.is_some());
    }

    // Join the mesh threads
    pub fn join_mesh(
        mut world: ResMut<World>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        // mut materials: ResMut<Assets<StandardMaterial>>,
        g_chunk_material: Res<GlobalChunkMaterial>,
    ) {
        let World {
            mesh_tasks,
            chunk_entities,
            ..
        } = world.as_mut();

        for (chunk_pos, task_option) in mesh_tasks.iter_mut() {
            let Some(mut task) = task_option.take() else {
                warn!("Someone modified a task");
                continue;
            };

            let Some(chunk_mesh) = block_on(future::poll_once(&mut task)) else {
                // Failed to poll, keep task alive
                *task_option = Some(task);
                continue;
            };

            let Some(mesh) = chunk_mesh else {
                continue;
            };

            // let vertices = mesh
            //     .vertices
            //     .iter()
            //     .map(|vertex| {
            //         [
            //             vertex.pos.x as f32,
            //             vertex.pos.y as f32,
            //             vertex.pos.z as f32,
            //         ]
            //     })
            //     .collect::<Vec<[f32; 3]>>();

            // let normals = mesh
            //     .vertices
            //     .iter()
            //     .map(|vertex| NORMALS_ARRAY[vertex.normal])
            //     .collect::<Vec<[f32; 3]>>();

            let bevy_mesh = Mesh::new(
                bevy::render::mesh::PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(
                ATTRIBUTE_VOXEL,
                mesh.vertices
                    .iter()
                    .cloned()
                    .map(|v| v.into())
                    .collect::<Vec<u32>>(),
            )
            // .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            // .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(Indices::U32(mesh.indices.clone()));

            let mesh_handle = meshes.add(bevy_mesh);

            if let Some(entity) = chunk_entities.get(chunk_pos) {
                // Remove any chunks at this position
                commands.entity(*entity).despawn();
            }

            // let hue = ((chunk_pos.x.unsigned_abs() as usize * CHUNK_SIZE
            //     + chunk_pos.y.unsigned_abs() as usize)
            //     * CHUNK_SIZE
            //     + chunk_pos.z.unsigned_abs() as usize) as f32
            //     * (360. / (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as f32);

            let chunk_entity = commands
                .spawn((
                    Aabb::from_min_max(Vec3::ZERO, Vec3::splat(CHUNK_SIZE as f32)),
                    MaterialMeshBundle {
                        transform: Transform::from_xyz(
                            (chunk_pos.x * CHUNK_SIZE as i32) as f32,
                            (chunk_pos.y * CHUNK_SIZE as i32) as f32,
                            (chunk_pos.z * CHUNK_SIZE as i32) as f32,
                        ),
                        mesh: mesh_handle,
                        material: g_chunk_material.0.clone(),
                        // material: materials.add(StandardMaterial {
                        //     base_color: Color::hsv(hue, 1., 1.),
                        //     ..default()
                        // }),
                        ..default()
                    },
                ))
                .id();

            chunk_entities.insert(*chunk_pos, chunk_entity);
        }

        mesh_tasks.retain(|(_chunk_pos, option_task)| option_task.is_some());
    }
}
