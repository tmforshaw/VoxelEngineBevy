use bevy::{
    asset::Assets,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::Face,
    },
};

use crate::{
    chunk::CHUNK_SIZE,
    chunk_mesh::{generate_indices, ChunkMesh, Direction, Quad},
    vertex::Vertex,
    voxel::{VoxelPos, VoxelType},
    world::World,
};

fn push_face(mesh: &mut ChunkMesh, dir: Direction, vertex_pos: VoxelPos, voxel_type: VoxelType) {
    let quad = Quad::from_dir(vertex_pos, dir);
    // println!("{vertex_pos:?}\t{:?}\t{:?}", quad.corners, quad.dir);

    for corner in quad.corners.into_iter() {
        mesh.vertices.push(Vertex::new(
            (corner[0], corner[1], corner[2]).into(),
            dir,
            voxel_type,
        ));
    }
}

pub fn build_chunk_mesh(
    world: Res<World>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (&chunk_pos, chunk) in world.chunks.iter() {
        let mut mesh = ChunkMesh::default();

        for index in 0..(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) {
            let voxel_pos = VoxelPos::from_index(index);

            let (current, back, left, down) = world.get_adjacent_voxels(voxel_pos, chunk_pos);

            let current = current.unwrap(); // Should never not be able to find the current voxel

            if current.voxel_type.is_solid() {
                if let Some(back) = back {
                    if !back.voxel_type.is_solid() {
                        push_face(&mut mesh, Direction::Back, voxel_pos, current.voxel_type)
                    }
                }

                if let Some(left) = left {
                    if !left.voxel_type.is_solid() {
                        push_face(&mut mesh, Direction::Left, voxel_pos, current.voxel_type)
                    }
                }

                if let Some(down) = down {
                    if !down.voxel_type.is_solid() {
                        push_face(&mut mesh, Direction::Down, voxel_pos, current.voxel_type)
                    }
                }
            } else {
                if let Some(back) = back {
                    if back.voxel_type.is_solid() {
                        push_face(&mut mesh, Direction::Front, voxel_pos, back.voxel_type)
                    }
                }

                if let Some(left) = left {
                    if left.voxel_type.is_solid() {
                        push_face(&mut mesh, Direction::Right, voxel_pos, left.voxel_type)
                    }
                }

                if let Some(down) = down {
                    if down.voxel_type.is_solid() {
                        push_face(&mut mesh, Direction::Up, voxel_pos, down.voxel_type);
                    }
                }
            }

            if !mesh.vertices.is_empty() {
                mesh.indices = generate_indices(mesh.vertices.len());

                let vertices = mesh
                    .vertices
                    .iter()
                    .map(|vertex| {
                        [
                            vertex.pos.x as f32,
                            vertex.pos.y as f32,
                            vertex.pos.z as f32,
                        ]
                    })
                    .collect::<Vec<[f32; 3]>>();

                let normals_arr = [
                    [-1.0, 0.0, 0.0], // Left
                    [1.0, 0.0, 0.0],  // Right
                    [0.0, -1.0, 0.0], // Down
                    [0.0, 1.0, 0.0],  // Up
                    [0.0, 0.0, -1.0], // Forward
                    [0.0, 0.0, 1.0],  // Back
                ];

                let normals = mesh
                    .vertices
                    .iter()
                    .map(|vertex| normals_arr[vertex.normal])
                    .collect::<Vec<[f32; 3]>>();

                let mesh_handle = meshes.add(
                    Mesh::new(
                        PrimitiveTopology::TriangleStrip,
                        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                    )
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
                    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                    .with_inserted_indices(Indices::U32(mesh.clone().indices)),
                );

                let hue = ((voxel_pos.x * CHUNK_SIZE + voxel_pos.y) * CHUNK_SIZE + voxel_pos.z)
                    as f32
                    * (360. / (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as f32);

                commands.spawn(PbrBundle {
                    mesh: mesh_handle,
                    material: materials.add(StandardMaterial {
                        base_color: Color::hsv(hue, 1., 1.),
                        cull_mode: Some(Face::Back),
                        ..Default::default()
                    }),
                    transform: Transform::from_xyz(
                        (chunk_pos.x * CHUNK_SIZE as i32) as f32,
                        (chunk_pos.y * CHUNK_SIZE as i32) as f32,
                        (chunk_pos.z * CHUNK_SIZE as i32) as f32,
                    ),
                    ..default()
                });

                mesh = ChunkMesh::default();
            }
        }
    }
}
