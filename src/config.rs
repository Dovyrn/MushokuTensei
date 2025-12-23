use bevy::prelude::{Mat4, Reflect, Resource, UVec4, Vec4};
use bevy::render::render_resource::ShaderType;
use bytemuck::{Pod, Zeroable};

#[derive(Resource)]
pub struct AppSettings {
    pub width: u32,
    pub height: u32,
    pub workgroup_size: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            width: 700,
            height: 512,
            workgroup_size: 8,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Reflect, ShaderType, Pod, Zeroable)]
pub struct Material {
    pub color : [f32;3],
    pub yield_strength : f32,
    pub density : f32,
    pub friction : f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color : [0.5, 0.5, 0.5],
            yield_strength : 100.0,
            density : 2500.0,
            friction : 0.5
        }
    }
}


#[repr(C)]
#[derive(ShaderType, Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct Node {
    pub packed_data: UVec4,
}

impl Node {
    pub fn new(child_ptr: u32, is_leaf: bool, pop_mask: u64) -> Self {
        let mut p0 = child_ptr << 2;
        if is_leaf {
            p0 |= 1;
        }

        Self {
            packed_data: UVec4::new(
                p0,
                (pop_mask & 0xFFFFFFFF) as u32,
                (pop_mask >> 32) as u32,
                0,
            ),
        }
    }
}

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, Pod, Zeroable)]
pub struct DispatchParams {
    pub inv_view_proj: Mat4,
    pub camera_origin: Vec4,
}

impl Default for DispatchParams {
    fn default() -> Self {
        Self {
            inv_view_proj: Mat4::IDENTITY,
            camera_origin: Vec4::ZERO,
        }
    }
}


pub struct Brick {
    pub voxels : [u8; 64]
}

impl Brick {
    pub fn pack_bits_64(&self) -> u64 {
        let mut mask = 0u64;
        for (i, &mat_id) in self.voxels.iter().enumerate() {
            if mat_id != 0 {
                mask |= 1 << i;
            }
        }
        mask
    }
}

