use crate::config::{AppSettings, DispatchParams, Node};
use crate::render::VoxelCamera;
use bevy::prelude::*;
use bevy::render::render_resource::{ShaderRef, StorageTextureAccess, TextureFormat};
use bevy_app_compute::prelude::*;
use crate::voxel_map::SvoStorage;
use crate::VOXEL_SHADER_ID;
#[derive(TypePath)]
pub struct VoxelShader;

impl ComputeShader for VoxelShader {
    fn shader() -> ShaderRef {
        let asset_id = AssetId::Uuid {
            uuid : VOXEL_SHADER_ID
        };
        Handle::<Shader>::Weak(asset_id).into()
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
            .add_uniform("pc", &DispatchParams::default())
            .add_storage("nodePool", &vec![Node::default(); 10000])
            .add_storage("leafData", &vec![0u32; 10000])
            .add_texture(
                "out_tex",
                width,
                height,
                TextureFormat::Rgba8Unorm,
                StorageTextureAccess::WriteOnly,
            )
            .add_pass::<VoxelShader>(
                [
                    (width + workgroup_size - 1) / workgroup_size,
                    (height + workgroup_size - 1) / workgroup_size,
                    1,
                ],
                &["pc", "nodePool", "leafData", "out_tex"],
            )
            .continuous()
            .build()
    }
}

pub fn handle_compute_params(
    mut worker: ResMut<AppComputeWorker<WriteTextureWorker>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<VoxelCamera>>,
    svo : Res<SvoStorage>,
) {
    let Ok((camera, transform)) = camera_q.single() else {
        return;
    };

    let projection = camera.clip_from_view();
    let camera_world_matrix = transform.compute_matrix();
    let view = camera_world_matrix.inverse();
    let inv_view_proj = (projection * view).inverse();

    let params = DispatchParams {
        inv_view_proj,
        camera_origin: Vec4::new(
            transform.translation().x,
            transform.translation().y,
            transform.translation().z,
            svo.tree_scale as f32,
        ),
    };

    worker.write("pc", &params);
}
