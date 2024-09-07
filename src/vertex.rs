use crate::{positions::WorldPos, voxel::VoxelType};

pub struct Vertex {
    pos: WorldPos,
    voxel_type: VoxelType,
}

pub struct VertexU32(u32);

impl Vertex {
    pub fn new(pos: WorldPos, voxel_type: VoxelType) -> Self {
        Self { pos, voxel_type }
    }

    pub fn from_u32(vertex: VertexU32) -> Self {
        let pos_mask = 0b111111u32; // 6 1s to mask each position component
        let voxel_type_mask = 0b1111111u32 << 25u32; // 7 1s, shifted, to mask voxel type

        let pos = WorldPos {
            x: (vertex.0 & pos_mask) as i32,
            y: ((vertex.0 & (pos_mask << 12u32)) >> 12u32) as i32,
            z: ((vertex.0 & (pos_mask << 6u32)) >> 6u32) as i32,
        };

        let voxel_type = ((vertex.0 & voxel_type_mask) >> 25u32).into();

        Self { pos, voxel_type }
    }

    pub fn to_u32(&self) -> VertexU32 {
        // Pos allocated 18 bits, 6 bits per component
        // Block type allocated 7 bits
        VertexU32(
            self.pos.x as u32
                | (self.pos.z as u32) << 6u32
                | (self.pos.y as u32) << 12u32
                | (self.voxel_type as u32) << 25u32,
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
