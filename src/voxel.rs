use crate::chunk::CHUNK_SIZE;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum VoxelType {
    None,
    Block,
}

#[derive(Copy, Clone, Debug)]
pub struct Voxel {
    pub kind: VoxelType,
}

impl Default for Voxel {
    fn default() -> Self {
        Self {
            kind: VoxelType::None,
        }
    }
}

impl From<VoxelType> for u32 {
    fn from(voxel_type: VoxelType) -> Self {
        match voxel_type {
            VoxelType::None => 0,
            VoxelType::Block => 1,
        }
    }
}

impl From<u32> for VoxelType {
    fn from(voxel_type: u32) -> Self {
        match voxel_type {
            0 => VoxelType::None,
            1 => VoxelType::Block,
            _ => panic!("Voxel type: {voxel_type} not recognised, so can't convert to VoxelType"),
        }
    }
}

#[derive(Copy, Clone)]
pub struct VoxelPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl VoxelPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub fn from_tuple(pos: (usize, usize, usize)) -> Self {
        assert!(
            pos.0 < CHUNK_SIZE && pos.1 < CHUNK_SIZE && pos.2 < CHUNK_SIZE,
            "x: {}, y: {}, z: {}",
            pos.0,
            pos.1,
            pos.2
        );

        Self {
            x: pos.0,
            y: pos.1,
            z: pos.2,
        }
    }

    pub fn to_tuple(self) -> (usize, usize, usize) {
        (self.x, self.y, self.z)
    }

    pub fn to_index(pos: VoxelPos) -> usize {
        pos.x + (pos.y + pos.z * CHUNK_SIZE) * CHUNK_SIZE
    }

    pub fn from_index(index: usize) -> VoxelPos {
        VoxelPos::new(
            index % CHUNK_SIZE,
            (index / CHUNK_SIZE) % CHUNK_SIZE,
            (index / (CHUNK_SIZE * CHUNK_SIZE)) % CHUNK_SIZE,
        )
    }
}

impl From<(usize, usize, usize)> for VoxelPos {
    fn from(pos: (usize, usize, usize)) -> Self {
        Self::from_tuple(pos)
    }
}

impl From<VoxelPos> for (usize, usize, usize) {
    fn from(chunk_pos: VoxelPos) -> Self {
        chunk_pos.to_tuple()
    }
}
