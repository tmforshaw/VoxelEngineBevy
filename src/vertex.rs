use crate::{chunk_mesh::Direction, positions::VoxelPos, voxel::VoxelType};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: VoxelPos,
    pub normal: usize, // Index of the normal
    pub voxel_type: VoxelType,
}

#[derive(Copy, Clone)]
pub struct VertexU32(u32);

impl VertexU32 {
    pub fn new(pos: VoxelPos, dir: Direction, voxel_type: VoxelType) -> Self {
        Vertex::new(pos, dir, voxel_type).into()
    }
}

impl Vertex {
    pub fn new(pos: VoxelPos, dir: Direction, voxel_type: VoxelType) -> Self {
        Self {
            pos,
            normal: dir.get_normal_index(),
            voxel_type,
        }
    }

    pub fn from_u32(vertex: VertexU32) -> Self {
        let pos_mask = 0b111111u32; // 6 1s to mask each position component
        let seven_bits_mask = 0b1111111u32; // 7 1s, shifted, to mask voxel type

        let pos = VoxelPos {
            x: (vertex.0 & pos_mask) as usize,
            y: ((vertex.0 & (pos_mask << 12u32)) >> 12u32) as usize,
            z: ((vertex.0 & (pos_mask << 6u32)) >> 6u32) as usize,
        };

        let normal = ((vertex.0 & (seven_bits_mask << 18u32)) >> 18u32) as usize;

        let voxel_type = ((vertex.0 & (seven_bits_mask << 25u32)) >> 25u32).into();

        Self {
            pos,
            normal,
            voxel_type,
        }
    }

    pub fn to_u32(&self) -> VertexU32 {
        // Pos allocated 18 bits, 6 bits per component
        // Normal allocated 3 bits
        // Block type allocated 11 bits
        VertexU32(
            self.pos.x as u32
                | (self.pos.y as u32) << 6u32
                | (self.pos.z as u32) << 12u32
                | (self.normal as u32) << 18u32
                | (self.voxel_type as u32) << 21u32,
        )
    }
}

impl From<Vertex> for VertexU32 {
    fn from(vertex: Vertex) -> Self {
        vertex.to_u32()
    }
}

impl From<VertexU32> for Vertex {
    fn from(vertex: VertexU32) -> Self {
        Self::from_u32(vertex)
    }
}

impl From<VertexU32> for u32 {
    fn from(vertex: VertexU32) -> Self {
        vertex.0
    }
}
