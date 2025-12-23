use bevy::prelude::{Mat4, Resource, UVec4, Vec4};
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

    pub fn new_empty() -> Self {
        Self {
            packed_data: UVec4::ZERO,
        }
    }

    pub fn set_leaf(&mut self, is_leaf: bool) {
        if is_leaf {
            self.packed_data.x |= 1;
        } else {
            self.packed_data.x &= !1;
        }
    }

    pub fn set_absolute_ptr(&mut self, is_abs: bool) {
        if is_abs {
            self.packed_data.x |= 2;
        } else {
            self.packed_data.x &= !2;
        }
    }

    pub fn set_child_ptr(&mut self, ptr: u32) {
        self.packed_data.x &= 3;
        self.packed_data.x |= ptr << 2;
    }

    pub fn get_child_ptr(&self) -> u32 {
        self.packed_data.x >> 2
    }

    pub fn set_pop_mask(&mut self, mask: u64) {
        self.packed_data.y = mask as u32;
        self.packed_data.z = (mask >> 32) as u32;
    }

    pub fn get_pop_mask(&self) -> u64 {
        (self.packed_data.y as u64) | ((self.packed_data.z as u64) << 32)
    }

    pub fn is_leaf(&self) -> bool {
        (self.packed_data.x & 1) != 0
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
