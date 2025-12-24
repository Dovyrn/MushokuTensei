use crate::compute::WriteTextureWorker;
use crate::config::AppSettings;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_asset::{RenderAssetUsages, RenderAssets};
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureUsages, TextureView,
};
use bevy::render::texture::GpuImage;
use bevy::window::WindowResized;
use bevy_app_compute::prelude::*;

#[derive(Resource, Clone, ExtractResource, Default)]
pub struct DisplayImage(pub Handle<Image>);

#[derive(Resource, Clone, ExtractResource)]
pub struct ComputeTransfer(pub TextureView);

#[derive(Component)]
pub struct VoxelCamera;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 1,
            ..default()
        },
    ));
    commands.spawn((
        Camera3d::default(),
        VoxelCamera,
        Transform::from_xyz(0.0, 0.0, 0.0) 
            .looking_at(Vec3::splat(512.0), Vec3::Y),
    ));
}

pub fn handle_resize(world: &mut World) {
    let resize_events = world.resource_mut::<Events<WindowResized>>();
    let events: Vec<WindowResized> = resize_events
        .get_cursor()
        .read(&resize_events)
        .cloned()
        .collect();

    if events.is_empty() && world.get_resource::<DisplayImage>().is_some() {
        return;
    }

    let (width, height) = {
        let mut settings = world.resource_mut::<AppSettings>();
        if let Some(last_event) = events.last() {
            settings.width = last_event.width as u32;
            settings.height = last_event.height as u32;
        }
        (settings.width, settings.height)
    };

    let new_image = create_gpu_image(width, height);
    let mut images = world.resource_mut::<Assets<Image>>();
    let handle = images.add(new_image);

    let mut sprite_query = world.query::<(Entity, &mut Sprite)>();
    let sprite_entity = sprite_query.iter_mut(world).next();

    if let Some((_, mut sprite)) = sprite_entity {
        sprite.image = handle.clone();
        sprite.custom_size = Some(Vec2::new(width as f32, height as f32));
    } else {
        world.spawn((
            Sprite {
                image: handle.clone(),
                custom_size: Some(Vec2::new(width as f32, height as f32)),
                ..default()
            },
            Transform::from_scale(Vec3::splat(1.0)),
        ));
    }

    world.insert_resource(DisplayImage(handle));

    let new_worker = WriteTextureWorker::build(world);
    world.insert_resource(new_worker);
}

pub fn create_gpu_image(width: u32, height: u32) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );

    image.texture_descriptor.usage |=
        TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    image
}

pub fn extract_compute_view(
    worker: Res<AppComputeWorker<WriteTextureWorker>>,
    mut commands: Commands,
) {
    if let Some(texture) = worker.get_texture("out_tex") {
        commands.insert_resource(ComputeTransfer(texture.view().clone()));
    }
}

pub fn link_compute_texture(
    display_image: Res<DisplayImage>,
    compute_transfer: Res<ComputeTransfer>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
) {
    if let Some(gpu_image) = gpu_images.get_mut(&display_image.0) {
        gpu_image.texture_view = compute_transfer.0.clone();
    }
}
