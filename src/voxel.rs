use crate::chunk::CHUNK_SIZE;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum VoxelType {
    Air,
    Block,
}

impl VoxelType {
    pub fn is_solid(&self) -> bool {
        match self {
            VoxelType::Block => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Voxel {
    pub voxel_type: VoxelType,
}

impl Voxel {
    pub fn new(voxel_type: VoxelType) -> Self {
        Self { voxel_type }
    }
}

impl Default for Voxel {
    fn default() -> Self {
        Self {
            voxel_type: VoxelType::Air,
        }
    }
}

impl From<VoxelType> for u32 {
    fn from(voxel_type: VoxelType) -> Self {
        match voxel_type {
            VoxelType::Air => 0,
            VoxelType::Block => 1,
        }
    }
}

impl From<u32> for VoxelType {
    fn from(voxel_type: u32) -> Self {
        match voxel_type {
            0 => VoxelType::Air,
            1 => VoxelType::Block,
            _ => panic!("Voxel type: {voxel_type} not recognised, so can't convert to VoxelType"),
        }
    }
}
