use crate::config::Node;
use crate::voxel_map::{CHUNK_SIZE, Voxel, VoxelMap};
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};

pub struct RenderClusterInfo {
    pub pop_mask: u64,
    pub chunk_roots: [Node; 64],
}

impl Default for RenderClusterInfo {
    fn default() -> Self {
        Self {
            pop_mask: 0,
            chunk_roots: [Node::new_empty(); 64],
        }
    }
}

pub fn get_cluster_key(pos: IVec3) -> u64 {
    let mut u = [pos.x as u64, pos.y as u64, pos.z as u64];

    // https://forceflow.be/2013/10/07/morton-encodingdecoding-through-bit-interleaving-implementations/
    for i in 0..3 {
        u[i] = (u[i] | (u[i] << 32)) & 0x000F00000000FFFF;
        u[i] = (u[i] | (u[i] << 16)) & 0x000F0000FF0000FF;
        u[i] = (u[i] | (u[i] << 8)) & 0x000F00F00F00F00F;
        u[i] = (u[i] | (u[i] << 4)) & 0x00C30C30C30C30C3;
    }

    u[0] | (u[2] << 2) | (u[1] << 4)
}

pub fn build_chunk_tree(
    map: &VoxelMap,
    node_pool: &mut Vec<Node>,
    leaf_data: &mut Vec<u32>,
    scale: i32,
    pos: IVec3,
) -> Node {
    let mut node = Node::new_empty();

    if scale == 2 {
        let mut mask: u64 = 0;
        let start_leaf_idx = leaf_data.len();

        for i in 0..64 {
            let cx = (i >> 0) & 3;
            let cy = (i >> 4) & 3;
            let cz = (i >> 2) & 3;

            let voxel_pos = pos + IVec3::new(cx, cy, cz);
            let voxel = map.get_voxel(voxel_pos);

            if !voxel.is_empty() {
                mask |= 1 << i;
                leaf_data.push(voxel.0);
            }
        }

        node.set_pop_mask(mask);
        if mask != 0 {
            node.set_leaf(true);
            node.set_child_ptr(start_leaf_idx as u32);
        }

        return node;
    }

    let next_scale = scale - 2;
    let mut children = Vec::new();
    let mut mask: u64 = 0;

    for i in 0..64 {
        let cx = (i >> 0) & 3;
        let cy = (i >> 4) & 3;
        let cz = (i >> 2) & 3;

        let child_pos = pos + (IVec3::new(cx, cy, cz) << next_scale);
        let child_node = build_chunk_tree(map, node_pool, leaf_data, next_scale, child_pos);

        if child_node.get_pop_mask() != 0 || child_node.is_leaf() {
            mask |= 1 << i;
            children.push(child_node);
        }
    }

    if mask != 0 {
        node.set_pop_mask(mask);
        node.set_child_ptr(node_pool.len() as u32);
        node_pool.extend(children);
    }

    node
}

pub fn build_tlas(
    clusters: &BTreeMap<u64, RenderClusterInfo>,
    node_pool: &mut Vec<Node>,
    root_scale: i32,
) -> Node {
    let mut layer_nodes: Vec<(u64, Node)> = Vec::new();

    for (&pos_key, cluster) in clusters {
        let start_ptr = node_pool.len() as u32;

        for i in 0..64 {
            if (cluster.pop_mask & (1 << i)) != 0 {
                node_pool.push(cluster.chunk_roots[i]);
            }
        }

        let mut node = Node::new_empty();
        node.set_child_ptr(start_ptr);
        node.set_pop_mask(cluster.pop_mask);

        layer_nodes.push((pos_key << (6 * 3), node));
    }

    let mut scale = 6 + 2;

    while scale < root_scale {
        let mut next_layer = Vec::new();
        let mut i = 0;
        while i < layer_nodes.len() {
            let start_ptr = node_pool.len() as u32;
            let mut node = Node::new_empty();
            node.set_child_ptr(start_ptr);

            let parent_mask = !0u64 << (scale * 3);
            let parent_pos = layer_nodes[i].0 & parent_mask;

            let mut mask: u64 = 0;
            while i < layer_nodes.len() {
                let (child_pos, child_node) = layer_nodes[i];
                if (child_pos ^ parent_pos) & parent_mask != 0 {
                    break;
                }

                let shift = (scale - 2) * 3;
                let idx = (child_pos >> shift) & 0x3F;

                mask |= 1 << idx;
                node_pool.push(child_node);
                i += 1;
            }

            node.set_pop_mask(mask);
            next_layer.push((parent_pos, node));
        }
        layer_nodes = next_layer;
        scale += 2;
    }

    if layer_nodes.is_empty() {
        Node::new_empty()
    } else {
        layer_nodes[0].1
    }
}
