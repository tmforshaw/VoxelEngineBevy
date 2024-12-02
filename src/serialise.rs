use std::{collections::HashMap, sync::Arc};

use crate::octree::{Node, NodeDataType, NodeWrappedType, MAX_CHILDREN};

#[derive(Debug, Clone)]
pub struct SerialNode {
    children: [u32; MAX_CHILDREN],
    data: u32,
}

impl Default for SerialNode {
    fn default() -> SerialNode {
        Self {
            children: [u32::MAX; MAX_CHILDREN],
            data: 0,
        }
    }
}

impl SerialNode {
    // Constructors

    pub fn new_branch() -> Self {
        Self::default()
    }

    pub fn new_leaf(data: NodeDataType) -> Self {
        Self {
            data: data.serialise(),
            ..Default::default()
        }
    }

    // Conversions between Node and SerialNode

    pub fn from_node(node: NodeWrappedType, map: Vec<(usize, NodeWrappedType)>) -> Self {
        let borrowed_node = node.clone();

        let mut new_node = if let Some(data) = borrowed_node.read().unwrap().clone().get_data() {
            SerialNode::new_leaf(data)
        } else {
            SerialNode::new_branch()
        };

        for (child_index, child) in borrowed_node
            .read()
            .unwrap()
            .clone()
            .children
            .iter()
            .enumerate()
        {
            if let Some(child) = child {
                let index = map
                    .iter()
                    .find_map(|(i, check_node)| {
                        if Arc::ptr_eq(child, check_node) {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .copied()
                    .unwrap();

                new_node.children[child_index] = index as u32;
            }
        }

        new_node
    }

    pub fn to_node(
        &self,
        node_ptr: NodeWrappedType,
        map: HashMap<usize, Option<NodeWrappedType>>,
    ) -> NodeWrappedType {
        todo!()
        // let data = NodeDataType::deserialise(self.data);

        // // Replace the indices with the smart pointers
        // let mut children = Some([const { None }; MAX_CHILDREN]);
        // for (i, child_index) in self.children.into_iter().enumerate() {
        //     children[i] = map[&(child_index as usize)].clone();
        // }

        // // Create a new node
        // let mut new_node = if let Some(data) = data {
        //     Node::new_leaf(data)
        // } else {
        //     Node::new_branch()
        // };
        // new_node.children = children;

        // // Replace the node
        // // *node_ptr.lock().unwrap() = new_node;
        // *node_ptr.write().unwrap() = new_node;

        // node_ptr
    }

    // Serialisation

    pub fn serialise(&self) -> u128 {
        let mut val = (self.data as u128) << 96;
        for (i, &child) in self.children.iter().enumerate() {
            val |= ((child as u128) & 0xFFF) << (((MAX_CHILDREN - i - 1) as u128) * 12);
        }

        val
    }

    pub fn deserialise(val: u128) -> Self {
        let data = (val >> 96) as u32;

        let mut children = [0; MAX_CHILDREN];
        (0..MAX_CHILDREN).for_each(|i| {
            children[i] = ((val >> (((MAX_CHILDREN - i - 1) as u128) * 12)) & 0xFFF) as u32;
        });

        let mut new_node = if (data & (1 << 31)) >> 31 > 0 {
            // Data exists if the most significant bit is 1
            SerialNode {
                data,
                ..Default::default()
            }
        } else {
            SerialNode::new_branch()
        };

        new_node.children = children;

        new_node
    }
}
