use bevy::math::IVec3;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Res, ResMut, Resource};
use crate::config::{Brick, Material, Node};

pub struct Sector {
    pub bricks: HashMap<u32, Brick>,
}

#[derive(Resource, Default)]
pub struct VoxelWorld {
    pub sectors: HashMap<IVec3, Sector>,
    pub palette: Vec<Material>,
}

#[derive(Resource, Default)]
pub struct SvoStorage {
    pub nodes: Vec<Node>,
    pub leaf_data: Vec<u32>,
    pub tree_scale: u32,
    pub dirty: bool,
}

pub fn update_svo_system(
    world: Res<VoxelWorld>,
    mut storage: ResMut<SvoStorage>,
) {
    if world.sectors.is_empty() {
        return;
    }

    world.generate_svo(&mut storage);

    storage.dirty = true;
}

pub fn get_morton_key(pos: IVec3) -> u64 {
    let mut x = pos.x as u64 & 0x1FFFFF;
    let mut y = pos.y as u64 & 0x1FFFFF;
    let mut z = pos.z as u64 & 0x1FFFFF;

    x = (x | (x << 32)) & 0x001F00000000FFFF;
    x = (x | (x << 16)) & 0x001F0000FF0000FF;
    x = (x | (x << 8))  & 0x100F00F00F00F00F;
    x = (x | (x << 4))  & 0x10C30C30C30C30C3;

    y = (y | (y << 32)) & 0x001F00000000FFFF;
    y = (y | (y << 16)) & 0x001F0000FF0000FF;
    y = (y | (y << 8))  & 0x100F00F00F00F00F;
    y = (y | (y << 4))  & 0x10C30C30C30C30C3;

    z = (z | (z << 32)) & 0x001F00000000FFFF;
    z = (z | (z << 16)) & 0x001F0000FF0000FF;
    z = (z | (z << 8))  & 0x100F00F00F00F00F;
    z = (z | (z << 4))  & 0x10C30C30C30C30C3;

    x | (z << 2) | (y << 4)
}

pub fn build_chunk_tree(
    world: &VoxelWorld,
    nodes: &mut Vec<Node>,
    leaf_data: &mut Vec<u32>,
    scale: i32,
    pos: IVec3,
) -> Option<Node> {
    if scale == 2 {
        if let Some(brick) = world.get_brick_at(pos) {
            let mask = brick.pack_bits_64();
            if mask == 0 { return None; }

            let child_ptr = leaf_data.len() as u32;
            for &mat_id in &brick.voxels {
                if mat_id != 0 { leaf_data.push(mat_id as u32); }
            }

            return Some(Node::new(child_ptr, true, mask));
        }
        return None;
    }

    let child_scale = scale - 2;
    let mut current_node_mask = 0u64;
    let mut children_results = Vec::with_capacity(64);

    for i in 0..64 {
        let child_offset = IVec3::new((i >> 0) & 3, (i >> 4) & 3, (i >> 2) & 3);
        if let Some(child_node) = build_chunk_tree(
            world, nodes, leaf_data,
            child_scale, pos + (child_offset << child_scale)
        ) {
            current_node_mask |= 1 << i;
            children_results.push(child_node);
        }
    }

    if current_node_mask == 0 { return None; }

    let child_start_ptr = nodes.len() as u32;
    nodes.extend(children_results);

    Some(Node::new(child_start_ptr, false, current_node_mask))
}

pub fn build_tlas(
    chunk_roots: Vec<(u64, Node)>,
    node_pool: &mut Vec<Node>,
    root_scale: i32,
) -> Node {
    if chunk_roots.is_empty() {
        return Node::default();
    }

    let mut layer_nodes = chunk_roots;
    let mut current_scale = 8;

    while current_scale <= root_scale || layer_nodes.len() > 1 {
        let mut next_layer = Vec::new();
        let mut i = 0;

        while i < layer_nodes.len() {
            let parent_mask_bits = (current_scale * 3) as u64;
            let parent_pos_key = layer_nodes[i].0 & (!0u64 << parent_mask_bits);

            let mut parent_pop_mask = 0u64;
            let child_start_ptr = node_pool.len() as u32;
            let mut children_to_push = Vec::new();

            while i < layer_nodes.len() && (layer_nodes[i].0 & (!0u64 << parent_mask_bits)) == parent_pos_key {
                let child_key = layer_nodes[i].0;
                let child_node = layer_nodes[i].1;

                let slot_index = (child_key >> ((current_scale - 2) * 3)) & 63;
                parent_pop_mask |= 1 << slot_index;

                children_to_push.push(child_node);
                i += 1;
            }

            node_pool.extend(children_to_push);

            let parent = Node::new(child_start_ptr, false, parent_pop_mask);
            next_layer.push((parent_pos_key, parent));
        }

        layer_nodes = next_layer;
        current_scale += 2;

        if current_scale > 30 { break; }
    }

    layer_nodes[0].1
}

impl VoxelWorld {
    pub fn generate_svo(&self, storage: &mut SvoStorage) {
        storage.nodes.clear();
        storage.leaf_data.clear();

        storage.nodes.push(Node::default());

        let mut chunk_roots = Vec::new();

        for (&sector_pos, _) in self.sectors.iter() {
            if let Some(root) = build_chunk_tree(self, &mut storage.nodes, &mut storage.leaf_data, 6, sector_pos * 64) {
                let morton_key = get_morton_key(sector_pos);
                chunk_roots.push((morton_key, root));
            }
        }

        if chunk_roots.is_empty() { return; }

        chunk_roots.sort_by_key(|k| k.0);

        let global_root = build_tlas(chunk_roots, &mut storage.nodes, storage.tree_scale as i32);
        storage.nodes[0] = global_root;
    }

    pub fn get_brick_at(&self, pos: IVec3) -> Option<&Brick> {
        let sector_pos = pos >> 6;
        if let Some(sector) = self.sectors.get(&sector_pos) {
            let local_pos: IVec3 = (pos >> 2) & 15;
            let brick_idx = (local_pos.x + local_pos.y * 16 + local_pos.z * 256) as u32;
            return sector.bricks.get(&brick_idx);
        }
        None
    }
}