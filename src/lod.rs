#[derive(Debug, Copy, Clone)]
pub enum Lod {
    L32,
    L16,
    L8,
    L4,
    L2,
}

impl Lod {
    // Voxels per axis
    pub fn size(&self) -> usize {
        match self {
            Lod::L32 => 32,
            Lod::L16 => 16,
            Lod::L8 => 8,
            Lod::L4 => 4,
            Lod::L2 => 2,
        }
    }

    // How much to multiply to reach next voxel
    pub fn jump_index(&self) -> usize {
        match self {
            Lod::L32 => 1,
            Lod::L16 => 2,
            Lod::L8 => 4,
            Lod::L4 => 8,
            Lod::L2 => 16,
        }
    }
}
