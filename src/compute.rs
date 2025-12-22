use bevy::prelude::*;
use bevy::render::render_resource::{ShaderRef, StorageTextureAccess, TextureFormat};
use bevy_app_compute::prelude::*;
use crate::config::AppSettings;

#[derive(TypePath)]
pub struct WriteShader;

impl ComputeShader for WriteShader {
    fn shader() -> ShaderRef {
        "shaders/write.wgsl".into()
    }
}

#[derive(Resource)]
pub struct WriteTextureWorker;

impl ComputeWorker for WriteTextureWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let (width, height, workgroup_size) = {
            let settings = world.resource::<AppSettings>();
            (settings.width, settings.height, settings.workgroup_size)
        };

        AppComputeWorkerBuilder::new(world)
            .add_texture(
                "output_texture",
                width,
                height,
                TextureFormat::Rgba8Unorm,
                StorageTextureAccess::WriteOnly,
            )
            .add_pass::<WriteShader>(
                [
                    width / workgroup_size,
                    height / workgroup_size,
                    1
                ],
                &["output_texture"],
            )
            .continuous()
            .build()
    }
}