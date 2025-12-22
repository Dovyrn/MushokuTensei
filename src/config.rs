use bevy::prelude::Resource;

#[derive(Resource)]
pub struct AppSettings {
    pub width : u32,
    pub height: u32,
    pub workgroup_size : u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            width : 700,
            height : 512,
            workgroup_size : 8,
        }
    }
}