// World Position Struct (For any voxel in the world)

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

use bevy::math::IVec3;

use crate::constants::CHUNK_SIZE;

#[derive(Copy, Clone, Debug)]
pub struct WorldPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    fn splat(val: i32) -> Self {
        Self::new(val, val, val)
    }

    pub fn to_voxel_pos(pos: Self) -> (VoxelPos, ChunkPos) {
        // Subtract CHUNK_SIZE / 2 before modulus so that negative chunks are rounded down to negative values (instead of rounded up to 0,0,0)
        // Add 0.5 before division so that before rounding, a value of 1/(2 * CHUNK_SIZE) is added, this makes the even rounding work for any chunk size
        let chunk_pos = (
            (((pos.x - CHUNK_SIZE as i32 / 2) as f32 + 0.5) / CHUNK_SIZE as f32).round_ties_even()
                as i32,
            (((pos.y - CHUNK_SIZE as i32 / 2) as f32 + 0.5) / CHUNK_SIZE as f32).round_ties_even()
                as i32,
            (((pos.z - CHUNK_SIZE as i32 / 2) as f32 + 0.5) / CHUNK_SIZE as f32).round_ties_even()
                as i32,
        )
            .into();

        // Have to add CHUNK_SIZE after the modulus to make it a true modulus function instead of just remainder (which includes negatives)
        let voxel_pos_i32 =
            ((pos % CHUNK_SIZE as i32) + WorldPos::splat(CHUNK_SIZE as i32)) % CHUNK_SIZE as i32;
        let voxel_pos = (
            voxel_pos_i32.x as usize,
            voxel_pos_i32.y as usize,
            voxel_pos_i32.z as usize,
        )
            .into();

        (voxel_pos, chunk_pos)
    }

    pub fn from_voxel_pos(voxel_pos: VoxelPos, chunk_pos: ChunkPos) -> Self {
        (
            voxel_pos.x as i32 + chunk_pos.x * CHUNK_SIZE as i32,
            voxel_pos.y as i32 + chunk_pos.y * CHUNK_SIZE as i32,
            voxel_pos.z as i32 + chunk_pos.z * CHUNK_SIZE as i32,
        )
            .into()
    }
}

impl From<(i32, i32, i32)> for WorldPos {
    fn from(pos: (i32, i32, i32)) -> Self {
        Self {
            x: pos.0,
            y: pos.1,
            z: pos.2,
        }
    }
}

impl Add<WorldPos> for WorldPos {
    type Output = WorldPos;

    fn add(self, rhs: WorldPos) -> Self::Output {
        (self.x + rhs.x, self.y + rhs.y, self.z + rhs.z).into()
    }
}

impl Rem<i32> for WorldPos {
    type Output = WorldPos;

    fn rem(self, rhs: i32) -> Self::Output {
        (self.x % rhs, self.y % rhs, self.z % rhs).into()
    }
}

// Chunk Position Struct (For the position of a chunk in the world)

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn splat(val: i32) -> Self {
        Self {
            x: val,
            y: val,
            z: val,
        }
    }

    pub fn from_vec3(pos: bevy::math::Vec3) -> Self {
        (pos.x as i32, pos.y as i32, pos.z as i32).into()
    }

    pub fn to_ivec3(&self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
    }

    pub fn from_tuple(pos: (i32, i32, i32)) -> Self {
        Self {
            x: pos.0,
            y: pos.1,
            z: pos.2,
        }
    }

    pub fn to_tuple(self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }

    pub fn distance_squared(&self, rhs: ChunkPos) -> u32 {
        ((self.x - rhs.x).pow(2) + (self.y - rhs.y).pow(2) + (self.z - rhs.z).pow(2)) as u32
    }
}

impl From<(i32, i32, i32)> for ChunkPos {
    fn from(pos: (i32, i32, i32)) -> Self {
        Self::from_tuple(pos)
    }
}

impl From<ChunkPos> for (i32, i32, i32) {
    fn from(chunk_pos: ChunkPos) -> Self {
        chunk_pos.to_tuple()
    }
}

impl Add<ChunkPos> for ChunkPos {
    type Output = ChunkPos;

    fn add(self, rhs: ChunkPos) -> Self::Output {
        (self.x + rhs.x, self.y + rhs.y, self.z + rhs.z).into()
    }
}

impl AddAssign<ChunkPos> for ChunkPos {
    fn add_assign(&mut self, rhs: ChunkPos) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub<ChunkPos> for ChunkPos {
    type Output = ChunkPos;

    fn sub(self, rhs: ChunkPos) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y, self.z - rhs.z).into()
    }
}

impl SubAssign<ChunkPos> for ChunkPos {
    fn sub_assign(&mut self, rhs: ChunkPos) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<i32> for ChunkPos {
    type Output = ChunkPos;

    fn mul(self, rhs: i32) -> Self::Output {
        (self.x * rhs, self.y * rhs, self.z * rhs).into()
    }
}

impl MulAssign<i32> for ChunkPos {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<i32> for ChunkPos {
    type Output = ChunkPos;

    fn div(self, rhs: i32) -> Self::Output {
        (self.x / rhs, self.y / rhs, self.z / rhs).into()
    }
}

impl DivAssign<i32> for ChunkPos {
    fn div_assign(&mut self, rhs: i32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

// Voxel Position Struct (For the position of a voxel within a chunk)

#[derive(Copy, Clone, Debug)]
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
        Self {
            x: pos.0,
            y: pos.1,
            z: pos.2,
        }
    }

    pub fn to_tuple(self) -> (usize, usize, usize) {
        (self.x, self.y, self.z)
    }

    pub fn to_index(&self) -> usize {
        self.x + (self.y + self.z * CHUNK_SIZE) * CHUNK_SIZE
    }

    pub fn from_index(index: usize) -> VoxelPos {
        VoxelPos::new(
            index % CHUNK_SIZE,
            (index / CHUNK_SIZE) % CHUNK_SIZE,
            (index / (CHUNK_SIZE * CHUNK_SIZE)) % CHUNK_SIZE,
        )
    }

    pub fn to_ivec3(&self) -> IVec3 {
        IVec3::new(self.x as i32, self.y as i32, self.z as i32)
    }

    pub fn from_ivec3(voxel_pos: IVec3) -> Self {
        Self::new(
            voxel_pos.x.max(0) as usize,
            voxel_pos.y.max(0) as usize,
            voxel_pos.z.max(0) as usize,
        )
    }

    pub fn to_i32(&self) -> (i32, i32, i32) {
        (self.x as i32, self.y as i32, self.z as i32)
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

pub fn index_to_chunk_pos_bounds(index: usize, bounds: u32) -> ChunkPos {
    (
        index as i32 % bounds as i32,
        (index as i32 / bounds as i32) % bounds as i32,
        index as i32 / (bounds * bounds) as i32,
    )
        .into()
}

pub fn chunk_pos_to_index_bounds(chunk_pos: ChunkPos, bounds: u32) -> usize {
    (chunk_pos.x % bounds as i32
        + (chunk_pos.y * bounds as i32)
        + chunk_pos.z * (bounds * bounds) as i32) as usize
}

impl Add<VoxelPos> for VoxelPos {
    type Output = VoxelPos;

    fn add(self, rhs: VoxelPos) -> Self::Output {
        (self.x + rhs.x, self.y + rhs.y, self.z + rhs.z).into()
    }
}

impl AddAssign<VoxelPos> for VoxelPos {
    fn add_assign(&mut self, rhs: VoxelPos) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub<VoxelPos> for VoxelPos {
    type Output = VoxelPos;

    fn sub(self, rhs: VoxelPos) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y, self.z - rhs.z).into()
    }
}

impl SubAssign<VoxelPos> for VoxelPos {
    fn sub_assign(&mut self, rhs: VoxelPos) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<usize> for VoxelPos {
    type Output = VoxelPos;

    fn mul(self, rhs: usize) -> Self::Output {
        (self.x * rhs, self.y * rhs, self.z * rhs).into()
    }
}

impl MulAssign<usize> for VoxelPos {
    fn mul_assign(&mut self, rhs: usize) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<usize> for VoxelPos {
    type Output = VoxelPos;

    fn div(self, rhs: usize) -> Self::Output {
        (self.x / rhs, self.y / rhs, self.z / rhs).into()
    }
}

impl DivAssign<usize> for VoxelPos {
    fn div_assign(&mut self, rhs: usize) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Rem<usize> for VoxelPos {
    type Output = VoxelPos;

    fn rem(self, rhs: usize) -> Self::Output {
        (self.x % rhs, self.y % rhs, self.z % rhs).into()
    }
}

impl RemAssign<usize> for VoxelPos {
    fn rem_assign(&mut self, rhs: usize) {
        self.x %= rhs;
        self.y %= rhs;
        self.z %= rhs;
    }
}
