use crate::{positions::WorldPos, vertex::VertexU32};

#[repr(u8)]
pub enum Direction {
    Left,
    Right,
    Backward,
    Forward,
    Up,
    Down,
}

#[derive(Default)]
pub struct ChunkMesh {
    pub vertices: Vec<VertexU32>,
    pub indices: Vec<u32>,
}

pub struct Quad {
    pub corners: [[i32; 3]; 4],
    pub dir: Direction,
}

impl Quad {
    pub fn from_dir(pos: WorldPos, dir: Direction) -> Self {
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
            Direction::Down => [
                [pos.x, pos.y, pos.z],
                [pos.x + 1, pos.y, pos.z],
                [pos.x + 1, pos.y, pos.z + 1],
                [pos.x, pos.y, pos.z + 1],
            ],
            Direction::Up => [
                [pos.x, pos.y, pos.z + 1],
                [pos.x + 1, pos.y, pos.z + 1],
                [pos.x + 1, pos.y, pos.z],
                [pos.x, pos.y, pos.z],
            ],
            Direction::Backward => [
                [pos.x, pos.y, pos.z],
                [pos.x, pos.y + 1, pos.z],
                [pos.x + 1, pos.y + 1, pos.z],
                [pos.x + 1, pos.y, pos.z],
            ],
            Direction::Forward => [
                [pos.x + 1, pos.y, pos.z],
                [pos.x + 1, pos.y + 1, pos.z],
                [pos.x, pos.y + 1, pos.z],
                [pos.x, pos.y, pos.z],
            ],
        };

        Self { corners, dir }
    }
}
