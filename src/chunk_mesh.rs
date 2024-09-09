use std::collections::VecDeque;

use bevy::math::IVec3;

use crate::{
    lod::Lod,
    positions::VoxelPos,
    vertex::{Vertex, VertexU32},
    voxel::VoxelType,
};

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
            Self::Left => 0,
            Self::Right => 1,
            Self::Back => 2,
            Self::Front => 3,
            Self::Up => 4,
            Self::Down => 5,
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

#[derive(Debug)]
pub struct GreedyQuad {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl GreedyQuad {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    pub fn append_vertices(
        &self,
        vertices: &mut Vec<VertexU32>,
        face_dir: FaceDir,
        axis: u32,
        lod: &Lod,
        ao: u32,
        voxel_type: VoxelType,
    ) {
        let jump = lod.jump_index();

        // Pack the ambient occlusion into the vertex
        let v1ao = (ao & 1) + ((ao >> 1) & 1) + ((ao >> 3) & 1);
        let v2ao = ((ao >> 3) & 1) + ((ao >> 6) & 1) + ((ao >> 7) & 1);
        let v3ao = ((ao >> 5) & 1) + ((ao >> 8) & 1) + ((ao >> 7) & 1);
        let v4ao = ((ao >> 1) & 1) + ((ao >> 2) & 1) + ((ao >> 5) & 1);

        let vertex_1 = VertexU32::new(
            face_dir.world_to_sample(axis, self.x, self.y) * jump,
            v1ao,
            face_dir.get_normal_index(),
            voxel_type,
        );

        let vertex_2 = VertexU32::new(
            face_dir.world_to_sample(axis, self.x + self.w, self.y) * jump,
            v2ao,
            face_dir.get_normal_index(),
            voxel_type,
        );

        let vertex_3 = VertexU32::new(
            face_dir.world_to_sample(axis, self.x + self.w, self.y + self.h) * jump,
            v3ao,
            face_dir.get_normal_index(),
            voxel_type,
        );

        let vertex_4 = VertexU32::new(
            face_dir.world_to_sample(axis, self.x, self.y + self.h) * jump,
            v4ao,
            face_dir.get_normal_index(),
            voxel_type,
        );

        let mut new_vertices = VecDeque::from([vertex_1, vertex_2, vertex_3, vertex_4]);

        // Change vertex order depending on face direction
        if face_dir.reverse_order() {
            // Keep the first vertex and reverse the others
            let first = new_vertices.split_off(1);
            first
                .into_iter()
                .rev()
                .for_each(|vertex| new_vertices.push_back(vertex));
        }

        // Anisotropy flip
        if (v1ao > 0) ^ (v3ao > 0) {
            // Right shift vec to swap triangle intersection angle
            let front = new_vertices.pop_front().unwrap();
            new_vertices.push_back(front);
        }

        vertices.extend(new_vertices);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FaceDir {
    Left,
    Right,
    Back,
    Front,
    Up,
    Down,
}

impl FaceDir {
    pub fn get_normal_index(&self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Back => 2,
            Self::Front => 3,
            Self::Up => 4,
            Self::Down => 5,
        }
    }

    // Direction to sample face culling
    pub fn sample_dir(&self) -> IVec3 {
        match self {
            Self::Left => IVec3::NEG_X,
            Self::Right => IVec3::X,
            Self::Back => IVec3::Z,
            Self::Front => IVec3::NEG_Z,
            Self::Up => IVec3::Y,
            Self::Down => IVec3::NEG_Y,
        }
    }

    // Offset input position with this face direction
    pub fn world_to_sample(&self, axis: u32, x: usize, y: usize) -> VoxelPos {
        match self {
            // Self::Left => (axis as usize, x, y).into(),
            // Self::Right => (axis as usize + 1, x, y).into(),
            // Self::Back => (x, y, axis as usize + 1).into(),
            // Self::Front => (x, y, axis as usize).into(),
            // Self::Up => (x, axis as usize + 1, y).into(),
            // Self::Down => (x, axis as usize, y).into(),
            FaceDir::Up => (x, axis as usize + 1, y).into(),
            FaceDir::Down => (x, axis as usize, y).into(),
            FaceDir::Left => (axis as usize, y, x).into(),
            FaceDir::Right => (axis as usize + 1, y, x).into(),
            FaceDir::Front => (x, y, axis as usize).into(),
            FaceDir::Back => (x, y, axis as usize + 1).into(),
        }
    }

    // Boolean to decide whether vertices need to be flipped to maintain counter-clockwise winding
    pub fn reverse_order(&self) -> bool {
        match self {
            Self::Left => false,
            Self::Right => true,
            Self::Back => false,
            Self::Front => true,
            Self::Up => true,
            Self::Down => false,
        }
    }
}
