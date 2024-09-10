use bevy::{
    math::IVec2,
    render::{mesh::MeshVertexAttribute, render_resource::VertexFormat},
};

use crate::positions::ChunkPos;

// Chunk constants

pub const CHUNK_LOAD_DISTANCE: u32 = 12;
pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_PADDED: usize = CHUNK_SIZE + 2;

pub const CHUNKS_FROM_MIDDLE_SIZE: usize = 3;

pub const CHUNK_VERTEX_SHADER: &str = "shaders/chunk.wgsl";
pub const CHUNK_FRAGMENT_SHADER: &str = "shaders/chunk.wgsl";

// Task constants

pub const MIN_THREADS: usize = 1;
pub const MAX_THREADS: usize = 16;

pub const MAX_DATA_TASKS: usize = 64;
pub const MAX_MESH_TASKS: usize = 64;
pub const MAX_CHUNK_LOADS: usize = 26000;

// World generation constants

pub const NOISE_SEED: u64 = 0;
pub const NOISE_FREQUENCY: f32 = 0.025;
pub const NOISE_HEIGHT_SCALE: f32 = 64.;

// Flycam constants

pub const FLYCAM_SENSITIVITY: f32 = 0.00015;
pub const FLYCAM_SPEED: f32 = 256.;

// Voxel constants

// A "high" random id should be used for custom attributes to ensure consistent sorting and avoid collisions with other attributes.
// See the MeshVertexAttribute docs for more info.
pub const ATTRIBUTE_VOXEL: MeshVertexAttribute =
    MeshVertexAttribute::new("Voxel", 696969696, VertexFormat::Uint32);

// Array constants

// const NORMALS_ARRAY: [[f32; 3]; 6] = [
//     [-1.0, 0.0, 0.0], // Left
//     [1.0, 0.0, 0.0],  // Right
//     [0.0, 0.0, 1.0],  // Back
//     [0.0, 0.0, -1.0], // Front
//     [0.0, 1.0, 0.0],  // Up
//     [0.0, -1.0, 0.0], // Down
// ];

// Adjacency array constants

pub const ADJACENT_CHUNK_DIRECTIONS: [ChunkPos; 27] = [
    ChunkPos { x: 0, y: 0, z: 0 },
    ChunkPos { x: 0, y: -1, z: -1 },
    ChunkPos { x: -1, y: 0, z: -1 },
    ChunkPos { x: -1, y: 0, z: 1 },
    ChunkPos { x: -1, y: -1, z: 0 },
    ChunkPos {
        x: -1,
        y: -1,
        z: -1,
    },
    ChunkPos { x: -1, y: 1, z: -1 },
    ChunkPos { x: -1, y: -1, z: 1 },
    ChunkPos { x: -1, y: 1, z: 1 },
    ChunkPos { x: 1, y: 0, z: -1 },
    ChunkPos { x: 1, y: -1, z: -1 },
    ChunkPos { x: 0, y: 1, z: -1 },
    ChunkPos { x: 1, y: 1, z: 1 },
    ChunkPos { x: 1, y: -1, z: 1 },
    ChunkPos { x: 1, y: 1, z: -1 },
    ChunkPos { x: 1, y: 1, z: 0 },
    ChunkPos { x: 0, y: 1, z: 1 },
    ChunkPos { x: 1, y: -1, z: 0 },
    ChunkPos { x: 0, y: -1, z: 1 },
    ChunkPos { x: 1, y: 0, z: 1 },
    ChunkPos { x: -1, y: 1, z: 0 },
    // von neumann neighbour
    ChunkPos { x: -1, y: 0, z: 0 },
    ChunkPos { x: 1, y: 0, z: 0 },
    ChunkPos { x: 0, y: -1, z: 0 },
    ChunkPos { x: 0, y: 1, z: 0 },
    ChunkPos { x: 0, y: 0, z: -1 },
    ChunkPos { x: 0, y: 0, z: 1 },
];

pub const ADJACENT_AO_DIRS: [IVec2; 9] = [
    IVec2::new(-1, -1),
    IVec2::new(-1, 0),
    IVec2::new(-1, 1),
    IVec2::new(0, -1),
    IVec2::new(0, 0),
    IVec2::new(0, 1),
    IVec2::new(1, -1),
    IVec2::new(1, 0),
    IVec2::new(1, 1),
];
