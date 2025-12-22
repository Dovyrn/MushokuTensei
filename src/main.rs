mod VoxelMap;

use bevy::window::WindowResolution;
use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_resource::{
            Extent3d, ShaderRef, StorageTextureAccess, TextureDimension, TextureFormat, TextureView,
        },
        texture::GpuImage,
    },
};
use bevy_app_compute::prelude::*;

const SIZE: (u32, u32) = (700, 512);
const WORKGROUP_SIZE: u32 = 8;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Mushoku Tensei".to_string(),
                    resolution: WindowResolution::new(SIZE.0 as f32, SIZE.1 as f32),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes_override: Some(true),
                ..default()
            }),
    )
    .add_plugins(AppComputePlugin)
    .add_plugins(AppComputeWorkerPlugin::<RedTextureWorker>::default())
    .add_plugins((
        ExtractResourcePlugin::<DisplayImage>::default(),
        ExtractResourcePlugin::<ComputeTransfer>::default(),
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (create_display_image, extract_compute_view));

    app.sub_app_mut(RenderApp)
        .add_systems(Render, link_compute_texture.in_set(RenderSet::Prepare));

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(TypePath)]
struct WriteRedShader;

impl ComputeShader for WriteRedShader {
    fn shader() -> ShaderRef {
        "shaders/write.wgsl".into()
    }
}

#[derive(Resource)]
struct RedTextureWorker;

impl ComputeWorker for RedTextureWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        AppComputeWorkerBuilder::new(world)
            .add_texture(
                "output_texture",
                SIZE.0,
                SIZE.1,
                TextureFormat::Rgba8Unorm,
                StorageTextureAccess::WriteOnly,
            )
            .add_pass::<WriteRedShader>(
                [SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1],
                &["output_texture"],
            )
            .continuous()
            .build()
    }
}

#[derive(Resource, Clone, ExtractResource)]
struct DisplayImage(Handle<Image>);

#[derive(Resource, Clone, ExtractResource)]
struct ComputeTransfer(TextureView);

fn create_display_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    display_image: Option<Res<DisplayImage>>,
) {
    if display_image.is_some() {
        return;
    }

    let image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 100, 0],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );

    let image_handle = images.add(image);

    commands.spawn((
        Sprite {
            image: image_handle.clone(),
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        Transform::from_scale(Vec3::splat(1.0)),
    ));

    commands.insert_resource(DisplayImage(image_handle));
    info!("Display image created!");
}

fn extract_compute_view(
    worker: Option<Res<AppComputeWorker<RedTextureWorker>>>,
    mut commands: Commands,
) {
    let Some(worker) = worker else { return };
    if let Some(texture) = worker.get_texture("output_texture") {
        commands.insert_resource(ComputeTransfer(texture.view().clone()));
    }
}

fn link_compute_texture(
    display_image: Option<Res<DisplayImage>>,
    compute_transfer: Option<Res<ComputeTransfer>>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
) {
    let (Some(display_img), Some(transfer)) = (display_image, compute_transfer) else {
        return;
    };

    if let Some(gpu_image) = gpu_images.get_mut(&display_img.0) {
        gpu_image.texture_view = transfer.0.clone();
    }
}
