use crate::{chunk_mesh::Direction, positions::VoxelPos, voxel::VoxelType};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: VoxelPos,
    pub ao: u32,
    pub normal: usize, // Index of the normal
    pub voxel_type: VoxelType,
}

#[derive(Copy, Clone)]
pub struct VertexU32(u32);

impl VertexU32 {
    pub fn new(pos: VoxelPos, ao: u32, normal_index: usize, voxel_type: VoxelType) -> Self {
        Vertex::new(pos, ao, normal_index, voxel_type).into()
    }
}

impl Vertex {
    pub fn new(pos: VoxelPos, ao: u32, normal_index: usize, voxel_type: VoxelType) -> Self {
        Self {
            pos,
            ao,
            normal: normal_index,
            voxel_type,
        }
    }

    pub fn from_u32(vertex: VertexU32) -> Self {
        let pos_mask = 0b111111u32; // 6 1s to mask each position component
        let three_bits_mask = 0b111u32; // 3 1s to mask ao and normal
        let eight_bits_mask = 0b11111111u32; // 8 1s to mask voxel type

        let pos = VoxelPos {
            x: (vertex.0 & pos_mask) as usize,
            y: ((vertex.0 & (pos_mask << 12u32)) >> 12u32) as usize,
            z: ((vertex.0 & (pos_mask << 6u32)) >> 6u32) as usize,
        };

        let ao = ((vertex.0 & (three_bits_mask << 18u32)) >> 18u32) as u32;
        let normal = ((vertex.0 & (three_bits_mask << 21u32)) >> 21u32) as usize;

        let voxel_type = ((vertex.0 & (eight_bits_mask << 24u32)) >> 24u32).into();

        Self {
            pos,
            normal,
            ao,
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
                | (self.ao as u32) << 18u32
                | (self.normal as u32) << 21u32
                | (self.voxel_type as u32) << 24u32,
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
