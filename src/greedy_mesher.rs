use std::collections::HashMap;

use bevy::{
    math::{IVec2, IVec3},
    pbr::generate_view_layouts,
};

use crate::{
    chunk::{CHUNK_SIZE, CHUNK_SIZE_PADDED},
    chunk_from_middle::{ChunksFromMiddle, CHUNKS_FROM_MIDDLE_SIZE},
    chunk_mesh::{generate_indices, ChunkMesh, FaceDir, GreedyQuad},
    lod::Lod,
    positions::{chunk_pos_to_index_bounds, VoxelPos},
    voxel::Voxel,
};

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

pub fn greedy_mesh_binary_plane(mut data: [u32; 32], lod_size: usize) -> Vec<GreedyQuad> {
    let mut greedy_quads = Vec::new();

    for row in 0..data.len() {
        let mut y = 0;

        while (y as usize) < lod_size {
            // Find the first solid block
            y += (data[row] >> y).trailing_zeros();
            if y as usize >= lod_size {
                // Reached the top of the data
                continue;
            }

            let height = (data[row] >> y).trailing_ones();

            // Convert height into (height)-many 1 bits
            let height_as_mask = u32::checked_shl(1, height).map_or(!0, |v| v - 1);
            let mask = height_as_mask << y;

            // Grow horizontally
            let mut width = 1;
            while row + width < lod_size {
                // Fetch the bits which span height
                let next_row_h = (data[row + width] >> y) & height_as_mask;

                if next_row_h != height_as_mask {
                    // Can't expand horizontally any more
                    break;
                }

                // Get rid of the bits which have been expanded into
                data[row + width] &= !mask;

                width += 1
            }

            greedy_quads.push(GreedyQuad::new(row, y as usize, width, height as usize));

            y += height;
        }
    }

    greedy_quads
}

pub fn build_chunk_mesh(chunks_from_middle: &ChunksFromMiddle, lod: Lod) -> Option<ChunkMesh> {
    if chunks_from_middle.are_all_voxels_same() {
        return None;
    }

    let mut mesh = ChunkMesh::default();
    let mut axis_cols = [[[0u64; CHUNK_SIZE_PADDED]; CHUNK_SIZE_PADDED]; 3]; // Solid binary for (x, y, z) axes
    let mut col_face_masks = [[[0u64; CHUNK_SIZE_PADDED]; CHUNK_SIZE_PADDED]; 6]; // The cull mask to perform greedy slicing

    // #[inline]
    fn add_voxel_to_axis_cols(
        voxel: &Voxel,
        x: usize,
        y: usize,
        z: usize,
        axis_cols: &mut [[[u64; CHUNK_SIZE_PADDED]; CHUNK_SIZE_PADDED]; 3],
    ) {
        if voxel.voxel_type.is_solid() {
            // x,z --- y axis
            axis_cols[0][z][x] |= 1 << y as u64;

            // y,z --- x axis
            axis_cols[1][y][z] |= 1 << x as u64;

            // x,y --- z axis
            axis_cols[2][y][x] |= 1 << z as u64;
        }
    }

    // Inner chunk voxels
    let chunk = &*chunks_from_middle.chunks
        [chunk_pos_to_index_bounds((1, 1, 1).into(), CHUNKS_FROM_MIDDLE_SIZE as u32)];
    assert!(chunk.len() == CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE || chunk.len() == 1);
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let i = match chunk.len() {
                    1 => 0,
                    _ => VoxelPos::new(x, y, z).to_index(),
                };

                add_voxel_to_axis_cols(&chunk[i], x + 1, y + 1, z + 1, &mut axis_cols);
            }
        }
    }

    // Neighbour chunk voxels
    // TODO Optimise these
    for z in [0, CHUNK_SIZE_PADDED - 1] {
        for y in 0..CHUNK_SIZE_PADDED {
            for x in 0..CHUNK_SIZE_PADDED {
                let voxel_pos = IVec3::new(x as i32, y as i32, z as i32) - IVec3::ONE;
                add_voxel_to_axis_cols(
                    chunks_from_middle.get_voxel(voxel_pos),
                    x,
                    y,
                    z,
                    &mut axis_cols,
                )
            }
        }
    }
    for z in 0..CHUNK_SIZE_PADDED {
        for y in [0, CHUNK_SIZE_PADDED - 1] {
            for x in 0..CHUNK_SIZE_PADDED {
                let voxel_pos = IVec3::new(x as i32, y as i32, z as i32) - IVec3::ONE;
                add_voxel_to_axis_cols(
                    chunks_from_middle.get_voxel(voxel_pos),
                    x,
                    y,
                    z,
                    &mut axis_cols,
                )
            }
        }
    }
    for z in 0..CHUNK_SIZE_PADDED {
        for x in [0, CHUNK_SIZE_PADDED - 1] {
            for y in 0..CHUNK_SIZE_PADDED {
                let voxel_pos = IVec3::new(x as i32, y as i32, z as i32) - IVec3::ONE;
                add_voxel_to_axis_cols(
                    chunks_from_middle.get_voxel(voxel_pos),
                    x,
                    y,
                    z,
                    &mut axis_cols,
                )
            }
        }
    }

    // Face culling
    for axis in 0..3 {
        for z in 0..CHUNK_SIZE_PADDED {
            for x in 0..CHUNK_SIZE_PADDED {
                // Set if current is solid and next is air
                let col = axis_cols[axis][z][x];

                col_face_masks[2 * axis][z][x] = col & !(col << 1); // Sample descending axis and set true when air meets solid
                col_face_masks[2 * axis + 1][z][x] = col & !(col >> 1); // Sample ascending axis and set true when air meets solid
            }
        }
    }

    // Greedy meshing planes for all 6 axes
    // key(voxel + ao) -> HashMap<axis(0-32), binary_plane>
    let mut data: [HashMap<u32, HashMap<u32, [u32; 32]>>; 6] = [
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    ];

    // Find faces and build binary planes based on the voxel+ao
    for axis in 0..6 {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // Skip using CHUNK_SIZE_PADDED by just adding 1 to x and 1 to z
                let mut col = col_face_masks[axis][z + 1][x + 1];

                // Remove right-most padding because it's invalid
                col >>= 1;

                // Remove left-most padding because it's invalid
                col &= !(1 << CHUNK_SIZE as u64);

                while col != 0 {
                    let y = col.trailing_zeros() as usize;

                    // Clear least significant, set, bit
                    col &= col - 1;

                    // Get the voxel position based on axis
                    let voxel_pos: VoxelPos = match axis {
                        0 | 1 => (x, y, z).into(), // Down, Up
                        2 | 3 => (y, z, x).into(), // Left, Right
                        _ => (x, z, y).into(),     // Front, Back
                    };

                    // Calculate ambient occlusion
                    let mut ao_index = 0;
                    for (ao_i, ao_offset) in ADJACENT_AO_DIRS.iter().enumerate() {
                        // AO is sampled based on axis (ascent or descent)
                        let ao_sample_offset = match axis {
                            0 => IVec3::new(ao_offset.x, -1, ao_offset.y), // Down
                            1 => IVec3::new(ao_offset.x, 1, ao_offset.y),  // Up
                            2 => IVec3::new(-1, ao_offset.y, ao_offset.x), // Left
                            3 => IVec3::new(1, ao_offset.y, ao_offset.x),  // Right
                            4 => IVec3::new(ao_offset.x, ao_offset.y, -1), // Front
                            _ => IVec3::new(ao_offset.x, ao_offset.y, 1),  // Back
                        };

                        let ao_voxel_pos = voxel_pos.to_ivec3() + ao_sample_offset;
                        let ao_voxel = chunks_from_middle.get_voxel(ao_voxel_pos);

                        if ao_voxel.voxel_type.is_solid() {
                            ao_index |= 1 << ao_i;
                        }
                    }

                    let current_voxel = chunks_from_middle.get_voxel_no_neighbour(voxel_pos);

                    // Can only greedy mesh same voxel types with same AO
                    let voxel_hash = ao_index | ((current_voxel.voxel_type as u32) << 9);
                    let data = data[axis]
                        .entry(voxel_hash)
                        .or_default()
                        .entry(y as u32)
                        .or_default();
                    data[x] |= 1 << z;
                }
            }
        }
    }

    // Time for greedy meshing
    let mut vertices = Vec::new();
    for (axis, voxel_ao_data) in data.into_iter().enumerate() {
        let face_dir = match axis {
            0 => FaceDir::Down,
            1 => FaceDir::Up,
            2 => FaceDir::Left,
            3 => FaceDir::Right,
            4 => FaceDir::Front,
            _ => FaceDir::Back,
        };

        for (voxel_ao, axis_plane) in voxel_ao_data.into_iter() {
            let ao = voxel_ao & 0b111111111; // 9 1s
            let voxel_type = (voxel_ao >> 9).into();

            for (axis_pos, plane) in axis_plane.into_iter() {
                let quads_from_axis = greedy_mesh_binary_plane(plane, lod.size());

                quads_from_axis.into_iter().for_each(|q| {
                    q.append_vertices(&mut vertices, face_dir, axis_pos, &Lod::L32, ao, voxel_type);
                })
            }
        }
    }

    mesh.vertices.extend(vertices);
    if mesh.vertices.is_empty() {
        None
    } else {
        mesh.indices = generate_indices(mesh.vertices.len());
        Some(mesh)
    }
}
