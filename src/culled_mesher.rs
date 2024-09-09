use crate::{
    chunk::CHUNK_SIZE,
    chunk_from_middle::ChunksFromMiddle,
    chunk_mesh::{generate_indices, ChunkMesh, Direction, Quad},
    positions::VoxelPos,
    vertex::{Vertex, VertexU32},
    voxel::VoxelType,
};

fn push_face(mesh: &mut ChunkMesh, dir: Direction, vertex_pos: VoxelPos, voxel_type: VoxelType) {
    let quad = Quad::from_dir(vertex_pos, dir);

    for corner in quad.corners.iter() {
        mesh.vertices.push(VertexU32::new(
            (corner[0], corner[1], corner[2]).into(),
            dir,
            voxel_type,
        ));
    }
}

pub fn build_chunk_mesh(chunks_from_middle: &ChunksFromMiddle) -> Option<ChunkMesh> {
    let mut mesh = ChunkMesh::default();

    for index in 0..(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) {
        let voxel_pos = VoxelPos::from_index(index);

        let (current, back, left, down) = chunks_from_middle.get_adjacent_voxels(voxel_pos);

        if current.voxel_type.is_solid() {
            if !left.voxel_type.is_solid() {
                push_face(&mut mesh, Direction::Left, voxel_pos, current.voxel_type)
            }

            if !back.voxel_type.is_solid() {
                push_face(&mut mesh, Direction::Back, voxel_pos, current.voxel_type)
            }

            if !down.voxel_type.is_solid() {
                push_face(&mut mesh, Direction::Down, voxel_pos, current.voxel_type)
            }
        } else {
            if left.voxel_type.is_solid() {
                push_face(&mut mesh, Direction::Right, voxel_pos, left.voxel_type)
            }

            if back.voxel_type.is_solid() {
                push_face(&mut mesh, Direction::Front, voxel_pos, back.voxel_type)
            }

            if down.voxel_type.is_solid() {
                push_face(&mut mesh, Direction::Up, voxel_pos, down.voxel_type);
            }
        }
    }

    if mesh.vertices.is_empty() {
        None
    } else {
        mesh.indices = generate_indices(mesh.vertices.len());
        Some(mesh)
    }
}
