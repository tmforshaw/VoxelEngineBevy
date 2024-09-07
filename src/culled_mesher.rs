use crate::{
    chunk::CHUNK_SIZE,
    chunk_mesh::{ChunkMesh, Direction, Quad},
    positions::WorldPos,
    vertex::Vertex,
    voxel::{VoxelPos, VoxelType},
};

fn push_face(mesh: &mut ChunkMesh, dir: Direction, vertex_pos: WorldPos, voxel_type: VoxelType) {
    let quad = Quad::from_dir(vertex_pos, dir);

    for corner in quad.corners.into_iter() {
        mesh.vertices
            .push(Vertex::new(vertex_pos, voxel_type).to_u32())
    }
}

pub fn build_chunk_mesh() -> ChunkMesh {
    let mesh = ChunkMesh::default();

    for index in 0..(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) {
        let local_pos = VoxelPos::from_index(index);
    }

    todo!()
}
