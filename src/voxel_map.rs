use bevy::prelude::*;
use std::collections::HashMap;

pub const CHUNK_SIZE: i32 = 64;
const BRICK_SIZE_LOG2: i32 = 3;
const BRICK_SIZE: i32 = 1 << BRICK_SIZE_LOG2;
const MASK_SIZE: i32 = 4;
const SECTOR_SIZE: i32 = BRICK_SIZE * MASK_SIZE;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Voxel(pub u32);

impl Voxel {
    pub const EMPTY: Self = Self(0);

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

pub struct Brick {
    pub data: Box<[Voxel; (BRICK_SIZE * BRICK_SIZE * BRICK_SIZE) as usize]>,
}

impl Default for Brick {
    fn default() -> Self {
        Self {
            data: Box::new([Voxel::EMPTY; (BRICK_SIZE * BRICK_SIZE * BRICK_SIZE) as usize]),
        }
    }
}

impl Brick {
    pub fn get(&self, x: i32, y: i32, z: i32) -> Voxel {
        let idx = Self::get_index(x, y, z);
        self.data[idx]
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, val: Voxel) {
        let idx = Self::get_index(x, y, z);
        self.data[idx] = val;
    }

    fn get_index(x: i32, y: i32, z: i32) -> usize {
        ((x & (BRICK_SIZE - 1))
            + (z & (BRICK_SIZE - 1)) * BRICK_SIZE
            + (y & (BRICK_SIZE - 1)) * BRICK_SIZE * BRICK_SIZE) as usize
    }
}

pub struct VoxelMap {
    pub bricks: HashMap<IVec3, Brick>,
    pub dirty_chunks: HashMap<IVec3, u64>,
}

impl Default for VoxelMap {
    fn default() -> Self {
        Self {
            bricks: HashMap::new(),
            dirty_chunks: HashMap::new(),
        }
    }
}

impl VoxelMap {
    pub fn get_brick(&self, pos: IVec3) -> Option<&Brick> {
        self.bricks.get(&pos)
    }

    pub fn get_brick_mut(&mut self, pos: IVec3) -> &mut Brick {
        self.bricks.entry(pos).or_default()
    }

    pub fn set_voxel(&mut self, pos: IVec3, val: Voxel) {
        let brick_pos = pos >> BRICK_SIZE_LOG2;
        let sub_pos = pos & (BRICK_SIZE - 1);

        let brick = self.get_brick_mut(brick_pos);
        brick.set(sub_pos.x, sub_pos.y, sub_pos.z, val);
    }

    pub fn get_voxel(&self, pos: IVec3) -> Voxel {
        let brick_pos = pos >> BRICK_SIZE_LOG2;
        let sub_pos = pos & (BRICK_SIZE - 1);

        if let Some(brick) = self.get_brick(brick_pos) {
            brick.get(sub_pos.x, sub_pos.y, sub_pos.z)
        } else {
            Voxel::EMPTY
        }
    }
}
