use crate::serialise::SerialNode;

use bevy::{
    prelude::*,
    render::{
        mesh::Indices,
        mesh::{PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
};
use tabled::Tabled;

use std::{
    collections::{HashMap, VecDeque},
    f32::consts::PI,
    fs::File,
    io::{Read, Write},
    sync::{Arc, RwLock},
};

const EXPONENT_MAX_CHILDREN: u32 = 3;
pub const MAX_CHILDREN: usize = 2_usize.pow(EXPONENT_MAX_CHILDREN);

// Generate log_2(MAX_CHILDREN) 1's, in the least significant bits of this mask
const INDEX_MASK: usize =
    usize::MAX - ((usize::MAX >> EXPONENT_MAX_CHILDREN) << EXPONENT_MAX_CHILDREN);

pub type NodeWrappedType = Arc<RwLock<Node>>;
pub type NodeChildArrayType = [Option<Arc<RwLock<Node>>>; MAX_CHILDREN];

#[allow(unused)]
#[derive(Debug, Copy, Clone)]
pub struct NodeDataType {
    pub colour: Color,
}

impl NodeDataType {
    pub fn new(colour: Color) -> Self {
        Self { colour }
    }

    // Serialise

    pub fn serialise(&self) -> u32 {
        let col = self.colour.to_linear();

        // Assumes value between 0 and 1
        fn pack_f32(val: f32) -> u8 {
            (u8::MAX as f32 * val) as u8
        }

        let extra_data = u8::MAX;

        ((extra_data as u32) << 24)
            | ((pack_f32(col.red) as u32) << 16)
            | ((pack_f32(col.green) as u32) << 8)
            | pack_f32(col.blue) as u32
    }

    pub fn deserialise(val: u32) -> Option<Self> {
        fn unpack_u8(val: u8) -> f32 {
            (val as f32) / (u8::MAX as f32)
        }

        let extra_data = val >> 24;

        if extra_data > 0 {
            Some(Self::new(Color::linear_rgb(
                unpack_u8(((val >> 16) & 0xFF) as u8),
                unpack_u8(((val >> 8) & 0xFF) as u8),
                unpack_u8((val & 0xFF) as u8),
            )))
        } else {
            None
        }
    }
}

// Node -------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Node {
    pub children: NodeChildArrayType,
    pub data: Option<NodeDataType>,
}

impl Default for Node {
    fn default() -> Node {
        Self {
            children: [const { None }; MAX_CHILDREN],
            data: None,
        }
    }
}

impl Node {
    // Constructors

    pub fn new_branch() -> Self {
        Self::default()
    }

    pub fn new_leaf(data: NodeDataType) -> Self {
        Self {
            data: Some(data),
            ..Default::default()
        }
    }

    // Getters

    pub fn get_children(&self) -> NodeChildArrayType {
        self.children.clone()
    }

    pub fn get_data(&self) -> Option<NodeDataType> {
        self.data
    }

    // Serialise

    pub fn serialise(&self) -> u32 {
        self.data
            .map_or((u8::MAX as u32) << 24, |data| data.serialise())
    }

    pub fn deserialise(val: u32) -> Option<Self> {
        NodeDataType::deserialise(val).map(Self::new_leaf)
    }

    // Tests

    #[allow(unused)]
    pub fn is_branch(&self) -> bool {
        // Count the children which exist
        self.children
            .iter()
            .fold(0, |acc, child| if child.is_some() { acc + 1 } else { acc })
            > 0
    }

    // Utility

    #[inline]
    pub fn wrap_with_cell(self) -> NodeWrappedType {
        Arc::new(RwLock::new(self))
    }
}

#[derive(Tabled, Debug)]
struct NodeInfo {
    index: u64,
    parent: u64,
    #[tabled(format("{:?}", self.data))]
    data: Option<NodeDataType>,
}

// Octree -----------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct Octree {
    root: NodeWrappedType,
    dim: usize,
}

impl Default for Octree {
    fn default() -> Self {
        Self {
            root: Node::default().wrap_with_cell(),
            dim: 0,
        }
    }
}

impl Octree {
    // Constructors

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_root(root: NodeWrappedType) -> Self {
        Self {
            root,
            ..Default::default()
        }
    }

    // Boundary Tests

    #[allow(unused)]
    fn is_pos_outside_bounds(&self, position: IVec3) -> bool {
        fn abs_svo(n: i32) -> u32 {
            if n < 0 {
                (-n - 1) as u32
            } else {
                n as u32
            }
        }

        // Optimisation because Dimension is a power of 2
        (abs_svo(position.x) | abs_svo(position.y) | abs_svo(position.z))
            >= 2_u32.pow(self.dim as u32)
    }

    // Inserting and Growing The Tree

    pub fn insert(&mut self, position: IVec3, data: NodeDataType) {
        // Grow to fit the position
        while self.is_pos_outside_bounds(position) {
            self.grow();
        }

        let index = self.world_pos_to_node_index(position);

        // Replace the node with a leaf which contains the data
        *self.traverse(index).write().unwrap() = Node::new_leaf(data);
    }

    // Grow the octree by one level
    fn grow(&mut self) {
        let current_root = self.root.read().unwrap().clone();

        // Copy the current root
        let mut new_root = Node::new_branch();
        new_root.children = current_root.clone().children.clone();

        // Move each child within a new node, on the opposite side to where it was in the original node
        for i in 0..MAX_CHILDREN {
            if let Some(node) = new_root.children[i].take() {
                let mut parent = if let Some(data) = node.read().unwrap().data {
                    Node::new_leaf(data)
                } else {
                    Node::new_branch()
                };

                // Move the node to the opposite index within the new octant
                parent.children[!i & INDEX_MASK] = Some(node);
                new_root.children[i] = Some(parent.wrap_with_cell());
            } else {
                new_root.children[i] = Some(Node::new_branch().wrap_with_cell());
            }
        }

        // Replace the root node with this new node
        self.root = new_root.wrap_with_cell();

        // Increment the size of the tree
        self.dim += 1;
    }

    // Search/Serialise Functions

    pub fn traverse(&mut self, index: u64) -> NodeWrappedType {
        let mut node = self.root.clone();

        // Travel through the tree, towards the index, creating nodes when necessary
        for i in (0..self.dim).rev() {
            // Process the index from the most significant to the least significant bits
            let idx =
                ((index >> (i * EXPONENT_MAX_CHILDREN as usize)) & (INDEX_MASK as u64)) as usize;

            // Borrow the node
            let borrowed_node = node.read().unwrap().clone();

            // If this node has a child in the position we need
            if let Some(new_node) = borrowed_node.clone().children[idx].clone() {
                node = new_node;
            } else {
                // Child doesn't exist, so create it

                // Copy the node
                let mut new_node = Node::new_branch();
                new_node.children = borrowed_node.children.clone();

                // Set the correct child to a new node (Branch or Leaf depending on if the node has data)
                new_node.children[idx].replace(
                    borrowed_node
                        .data
                        .map_or_else(Node::new_branch, Node::new_leaf)
                        .wrap_with_cell(),
                );

                // Replace the node with the new node
                // node.replace(new_node);
                *node.write().unwrap() = new_node;

                // Set the next node to the correct child of the current node
                node = borrowed_node.clone().children[idx].clone().unwrap();
            }

            // Exit once the index has been processed
            if idx == 0 {
                break;
            }
        }

        // Return the node which was found
        node
    }

    #[allow(unused)]
    fn full_traversal(&self, breadth_first: bool) -> Vec<u128> {
        let mut stack = VecDeque::from([(0, Some(self.root.clone()))]);
        let mut node_infos = Vec::new();

        let mut serialisable = Vec::from([self.root.clone()]);

        // Perform a breadth-first search of the tree
        while !stack.is_empty() {
            // Pop from the front of the stack, unwrapping the node
            let popped_node = if breadth_first {
                stack.pop_front()
            } else {
                stack.pop_back()
            };

            let (index, current) = if let (index, Some(current)) = popped_node.unwrap() {
                (index, current)
            } else {
                continue;
            };

            let current_node = current.read().unwrap().clone();

            // Add the node information to the Vec
            node_infos.push(NodeInfo {
                index: index as u64,
                parent: index as u64 >> EXPONENT_MAX_CHILDREN,
                data: current_node.data,
            });

            let current_children = current_node.children.clone();

            serialisable.push(current);

            // Index the children, with enough space to fit MAX_CHILDREN for each 1 of index
            let mut indexed_children = current_children
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, node)| (i + index * MAX_CHILDREN, node))
                .collect::<VecDeque<_>>();

            stack.append(&mut indexed_children);
        }

        // // Print the table
        // println!("{}", Table::new(node_infos));

        // Generate a map between indices and pointers
        let node_map = serialisable
            .clone()
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>();

        // Serialise the nodes using a map between indices and pointers
        serialisable
            .into_iter()
            .map(|node| SerialNode::from_node(node, node_map.clone()).serialise())
            .collect::<Vec<_>>()
    }

    #[allow(unused)]
    pub fn breadth_first(&self) -> Vec<u128> {
        self.full_traversal(true)
    }

    #[allow(unused)]
    pub fn depth_first(&self) -> Vec<u128> {
        self.full_traversal(false)
    }

    pub fn serialise(&self) -> Vec<u128> {
        self.depth_first()
    }

    pub fn deserialise(serial: Vec<u128>) -> Self {
        // Unpack the integers into SerialNodes
        let serial_nodes = serial
            .into_iter()
            .map(SerialNode::deserialise)
            .collect::<Vec<_>>();

        // Create a list of pointers to nodes (To replace the integer pointers)
        let mut pointers = Vec::with_capacity(serial_nodes.len());
        for _ in 0..serial_nodes.len() {
            pointers.push(Some(Node::new_branch().wrap_with_cell()));
        }

        // Create a hashmap between integer indices and the pointers to nodes (Adding an entry for a None type)
        let mut map = pointers
            .clone()
            .into_iter()
            .enumerate()
            .collect::<HashMap<usize, Option<NodeWrappedType>>>();
        map.insert(usize::MAX & 0xFFF, None);

        // Convert the SerialNode types into Node types
        let nodes = serial_nodes
            .iter()
            .zip(pointers)
            .map(|(node, node_ptr)| node.to_node(node_ptr.unwrap(), map.clone()))
            .collect::<Vec<_>>();

        // Create an octree with the new root node
        Octree::with_root(nodes[0].clone())
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let serial = self.serialise();

        let bytes = serial.iter().fold(Vec::new(), |mut acc, chunk| {
            // Split the data into bytes
            let mut bytes = (0..12).map(|i| ((chunk >> (8 * i)) & 0xFF) as u8).collect();

            acc.append(&mut bytes);

            acc
        });

        // Create the file
        let mut file = File::create(filename)?;
        file.write_all(bytes.as_slice())?;

        Ok(())
    }

    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Open the file
        let mut file = File::open(filename)?;

        // Get the contents
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        let serial = contents.chunks(12).fold(Vec::new(), |mut acc, chunk| {
            // Combine the bytes into 128-bit chunks
            acc.push((0..12).fold(0, |acc, i| acc | (chunk[i] as u128) << (8 * i)));

            acc
        });

        Ok(Self::deserialise(serial))
    }

    // Display Functions

    pub fn get_node_mesh(
        line_length: f32,
        line_radius: f32,
        centre: Vec3,
        node_index: u64,
    ) -> Mesh {
        let mut positions = Vec::<[f32; 3]>::new();
        let mut uvs = Vec::<[f32; 2]>::new();
        let mut normals = Vec::<[f32; 3]>::new();
        let mut indices = Vec::<u32>::new();

        // Offset the centre of the node depending on its index
        let mut offset = Vec3::splat(0.);
        let mut idx = node_index;
        let mut i = 0;
        while idx != 0 {
            offset += (line_length / 2f32.powi(i + 1))
                * Vec3::new(
                    (idx & 0b1) as f32 - 0.5,
                    ((idx >> 1) & 0b1) as f32 - 0.5,
                    ((idx >> 2) & 0b1) as f32 - 0.5,
                );

            i += 1;
            idx >>= 3;
        }
        let depth = i; // The depth of this node is the maximum of i

        // A line mesh which has been scaled depending on this node's depth
        let line_mesh = Mesh::from(Capsule3d::new(
            line_radius / 2f32.powi(depth),
            line_length / 2f32.powi(depth),
        ));

        // Create a line mesh for each of the cube edges
        let mut mesh;
        for axis in 0..3 {
            for x in [
                -line_length / 2f32.powi(depth + 1),
                line_length / 2f32.powi(depth + 1),
            ] {
                for z in [
                    -line_length / 2f32.powi(depth + 1),
                    line_length / 2f32.powi(depth + 1),
                ] {
                    // Vary the axes in such a way that a cube is formed
                    let pos = match axis {
                        0 => Vec3::new(z, 0., x),
                        1 => Vec3::new(x, z, 0.),
                        2 => Vec3::new(0., x, z),
                        _ => unreachable!(),
                    } - centre
                        - offset;

                    // Which axis to rotate around depending on the axis variable
                    let rotation_axis = match axis {
                        0 => Vec3::Y,
                        1 => Vec3::X,
                        2 => Vec3::Z,
                        _ => unreachable!(),
                    };

                    // Translate and rotate the base mesh to the correct position and angle
                    let transform = Transform::from_xyz(pos.x, pos.y, pos.z)
                        .with_rotation(Quat::from_axis_angle(rotation_axis, PI / 2.));
                    mesh = line_mesh.clone().transformed_by(transform);

                    // Get the vertex attributes from this mesh
                    let (mesh_pos, mesh_uv, mesh_norm, mesh_indices) = if let (
                        Some(VertexAttributeValues::Float32x3(mesh_pos)),
                        Some(VertexAttributeValues::Float32x2(mesh_uv)),
                        Some(VertexAttributeValues::Float32x3(mesh_norm)),
                        Some(mesh_indices),
                    ) = (
                        mesh.attribute(Mesh::ATTRIBUTE_POSITION),
                        mesh.attribute(Mesh::ATTRIBUTE_UV_0),
                        mesh.attribute(Mesh::ATTRIBUTE_NORMAL),
                        mesh.indices(),
                    ) {
                        (
                            mesh_pos,
                            mesh_uv,
                            mesh_norm,
                            mesh_indices
                                .iter()
                                .map(|i| i + positions.len())
                                .collect::<Vec<usize>>(),
                        )
                    } else {
                        eprintln!("Could not get vertex positions from mesh:\nPos:\t{pos:?}\nRot:\t{rotation_axis:?}");
                        unreachable!();
                    };

                    positions.extend(mesh_pos);
                    uvs.extend(mesh_uv);
                    normals.extend(mesh_norm);
                    indices.extend(mesh_indices.iter().map(|&i| i as u32));
                }
            }
        }

        // Create a new mesh from all of these components
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
    }

    pub fn draw_octree(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        oct: Res<Octree>,
    ) {
        let line_length = 10.0;
        let line_radius = 0.025 * line_length / 2.;
        let centre = Vec3::splat(0.);

        let material1 = materials.add(Color::rgb_u8(124, 144, 255));
        let material2 = materials.add(Color::rgb_u8(255, 124, 144));
        let material3 = materials.add(Color::rgb_u8(144, 255, 124));

        // Spawn a new node  mesh into the world
        commands.spawn(PbrBundle {
            mesh: meshes.add(Self::get_node_mesh(line_length, line_radius, centre, 0)),
            material: material1.clone(),
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(Self::get_node_mesh(line_length, line_radius, centre, 1)),
            material: material2.clone(),
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(Self::get_node_mesh(line_length, line_radius, centre, 2)),
            material: material3.clone(),
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(Self::get_node_mesh(line_length, line_radius, centre, 3)),
            material: material1.clone(),
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(Self::get_node_mesh(line_length, line_radius, centre, 4)),
            material: material2.clone(),
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(Self::get_node_mesh(line_length, line_radius, centre, 5)),
            material: material3.clone(),
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(Self::get_node_mesh(line_length, line_radius, centre, 6)),
            material: material1.clone(),
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(Self::get_node_mesh(line_length, line_radius, centre, 7)),
            material: material2.clone(),
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(Self::get_node_mesh(line_length, line_radius, centre, 8)),
            material: material3.clone(),
            ..default()
        });
    }

    // Utility Functions

    fn normalise_pos(&self, world_pos: IVec3) -> (u32, u32, u32) {
        unsafe {
            (
                *(((&(world_pos.x + i32::pow(2, self.dim as u32))) as *const i32) as *const u32),
                *(((&(world_pos.y + i32::pow(2, self.dim as u32))) as *const i32) as *const u32),
                *(((&(world_pos.z + i32::pow(2, self.dim as u32))) as *const i32) as *const u32),
            )
        }
    }

    fn world_pos_to_node_index(&self, world_pos: IVec3) -> u64 {
        let p_norm = self.normalise_pos(world_pos);
        Self::interleave_three(p_norm)
    }

    // fn node_index_to_world_pos(index: u64) -> (i32, i32, i32) {}

    fn interleave_two(input: u32) -> u64 {
        const NUM_INPUTS: usize = 3;
        const MASKS: [u64; 5] = [
            0x9249_2492_4924_9249,
            0x30C3_0C30_C30C_30C3,
            0xF00F_00F0_0F00_F00F,
            0x00FF_0000_FF00_00FF,
            0xFFFF_0000_0000_FFFF,
        ];

        let mut n: u64 = input as u64;
        for i in (0..5).rev() {
            let shift = (NUM_INPUTS - 1) * (1 << i);
            n |= n << shift;
            n &= MASKS[i];
        }

        n
    }

    fn interleave_three((x, y, z): (u32, u32, u32)) -> u64 {
        (Self::interleave_two(x) << 2) | (Self::interleave_two(y) << 1) | Self::interleave_two(z)
    }
}
