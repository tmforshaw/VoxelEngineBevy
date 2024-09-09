use crate::{positions::VoxelPos, vertex::Vertex, vertex::VertexU32};

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Direction {
    Left,
    Right,
    Back,
    Front,
    Up,
    Down,
}

impl Direction {
    pub fn get_normal_index(&self) -> usize {
        match self {
            Direction::Left => 0,
            Direction::Right => 1,
            Direction::Back => 2,
            Direction::Front => 3,
            Direction::Up => 4,
            Direction::Down => 5,
        }
    }
}

#[derive(Default, Clone)]
pub struct ChunkMesh {
    // pub vertices: Vec<Vertex>,
    pub vertices: Vec<VertexU32>,
    pub indices: Vec<u32>,
}

pub struct Quad {
    pub corners: [[usize; 3]; 4],
    pub dir: Direction,
}

impl Quad {
    pub fn from_dir(pos: VoxelPos, dir: Direction) -> Self {
        let corners = match dir {
            Direction::Left => [
                [pos.x, pos.y, pos.z],
                [pos.x, pos.y, pos.z + 1],
                [pos.x, pos.y + 1, pos.z + 1],
                [pos.x, pos.y + 1, pos.z],
            ],
            Direction::Right => [
                [pos.x, pos.y + 1, pos.z],
                [pos.x, pos.y + 1, pos.z + 1],
                [pos.x, pos.y, pos.z + 1],
                [pos.x, pos.y, pos.z],
            ],
            Direction::Back => [
                [pos.x, pos.y, pos.z],
                [pos.x, pos.y + 1, pos.z],
                [pos.x + 1, pos.y + 1, pos.z],
                [pos.x + 1, pos.y, pos.z],
            ],
            Direction::Front => [
                [pos.x + 1, pos.y, pos.z],
                [pos.x + 1, pos.y + 1, pos.z],
                [pos.x, pos.y + 1, pos.z],
                [pos.x, pos.y, pos.z],
            ],
            Direction::Up => [
                [pos.x, pos.y, pos.z + 1],
                [pos.x + 1, pos.y, pos.z + 1],
                [pos.x + 1, pos.y, pos.z],
                [pos.x, pos.y, pos.z],
            ],
            Direction::Down => [
                [pos.x, pos.y, pos.z],
                [pos.x + 1, pos.y, pos.z],
                [pos.x + 1, pos.y, pos.z + 1],
                [pos.x, pos.y, pos.z + 1],
            ],
        };

        Self { corners, dir }
    }
}

// generate a vec of indices
// assumes vertices are made of quads, and counter clockwise ordered
pub fn generate_indices(vertex_count: usize) -> Vec<u32> {
    let mut indices = Vec::with_capacity((vertex_count * 6) / 4);
    for vert_index in (0..vertex_count as u32).step_by(4) {
        indices.append(&mut vec![
            vert_index,
            vert_index + 1,
            vert_index + 2,
            vert_index,
            vert_index + 2,
            vert_index + 3,
        ]);
    }
    indices
}
