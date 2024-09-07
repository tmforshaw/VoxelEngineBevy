use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

use crate::{
    chunk::{Chunk, CHUNK_SIZE},
    positions::{ChunkPos, WorldPos},
    voxel::{VoxelPos, VoxelType},
};

#[derive(Resource, Default)]
pub struct World {
    pub chunks: HashMap<ChunkPos, Arc<Mutex<Chunk>>>,
}

impl World {
    pub fn new_with(voxels_at: Vec<WorldPos>) -> Self {
        let mut world = World::default();

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
                    if chunk[voxel_index].kind != VoxelType::None {
                        let mesh_handle = meshes.add(generate_mesh());

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
                let (chunk_pos, voxel_pos) = WorldPos::to_voxel_pos(world_pos);

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
}

fn generate_mesh() -> Mesh {
    // Each array is an [x, y, z] coordinate in local space.
    // The camera coordinate space is right-handed x-right, y-up, z-back. This means "forward" is -Z.
    // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
    // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
    let triangle_vec = vec![
        // top (facing towards +y)
        [-0.5, 0.5, -0.5], // vertex with index 0
        [0.5, 0.5, -0.5],  // vertex with index 1
        [0.5, 0.5, 0.5],   // etc. until 23
        [-0.5, 0.5, 0.5],
        // bottom   (-y)
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
        // right    (+x)
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5], // This vertex is at the same position as vertex with index 2, but they'll have different UV and normal
        [0.5, 0.5, -0.5],
        // left     (-x)
        [-0.5, -0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
        // back     (+z)
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, -0.5, 0.5],
        // forward  (-z)
        [-0.5, -0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, -0.5, -0.5],
    ];

    // Set-up UV coordinates to point to the upper (V < 0.5), "dirt+grass" part of the texture.
    // Take a look at the custom image (assets/textures/array_texture.png)
    // so the UV coords will make more sense
    // Note: (0.0, 0.0) = Top-Left in UV mapping, (1.0, 1.0) = Bottom-Right in UV mapping
    let uv_coords = vec![
        // Assigning the UV coords for the top side.
        [0.0, 0.2],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 0.2],
        // Assigning the UV coords for the bottom side.
        [0.0, 0.45],
        [0.0, 0.25],
        [1.0, 0.25],
        [1.0, 0.45],
        // Assigning the UV coords for the right side.
        [1.0, 0.45],
        [0.0, 0.45],
        [0.0, 0.2],
        [1.0, 0.2],
        // Assigning the UV coords for the left side.
        [1.0, 0.45],
        [0.0, 0.45],
        [0.0, 0.2],
        [1.0, 0.2],
        // Assigning the UV coords for the back side.
        [0.0, 0.45],
        [0.0, 0.2],
        [1.0, 0.2],
        [1.0, 0.45],
        // Assigning the UV coords for the forward side.
        [0.0, 0.45],
        [0.0, 0.2],
        [1.0, 0.2],
        [1.0, 0.45],
    ];

    // For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
    // the surface.
    // Normals are required for correct lighting calculations.
    // Each array represents a normalized vector, which length should be equal to 1.0.
    let normals = vec![
        // Normals for the top side (towards +y)
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        // Normals for the bottom side (towards -y)
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        // Normals for the right side (towards +x)
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        // Normals for the left side (towards -x)
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        // Normals for the back side (towards +z)
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Normals for the forward side (towards -z)
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
    ];

    // Create the triangles out of the 24 vertices we created.
    // To construct a square, we need 2 triangles, therefore 12 triangles in total.
    // To construct a triangle, we need the indices of its 3 defined vertices, adding them one
    // by one, in a counter-clockwise order (relative to the position of the viewer, the order
    // should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
    // Read more about how to correctly build a mesh manually in the Bevy documentation of a Mesh,
    // further examples and the implementation of the built-in shapes.
    let indices = vec![
        0, 3, 1, 1, 3, 2, // triangles making up the top (+y) facing side.
        4, 5, 7, 5, 6, 7, // bottom (-y)
        8, 11, 9, 9, 11, 10, // right (+x)
        12, 13, 15, 13, 14, 15, // left (-x)
        16, 19, 17, 17, 19, 18, // back (+z)
        20, 21, 23, 21, 22, 23, // forward (-z)
    ];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, triangle_vec)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv_coords)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}
