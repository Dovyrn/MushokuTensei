mod compute;
mod config;
mod render;
mod VoxelMap;

use bevy::prelude::*;
use bevy::render::{Render, RenderApp, RenderSet, extract_resource::ExtractResourcePlugin};
use bevy::window::WindowResolution;
use bevy_app_compute::prelude::*;

use crate::compute::{WriteTextureWorker, handle_compute_params};
use crate::config::AppSettings;
use crate::render::*;

fn main() {
    let mut app = App::new();
    let settings = AppSettings::default();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Mushoku Tensei".to_string(),
                    resolution: WindowResolution::new(
                        settings.width as f32,
                        settings.height as f32,
                    ),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes_override: Some(true),
                ..default()
            }),
    )
    .insert_resource(settings)
    .add_plugins(AppComputePlugin)
    .add_plugins(AppComputeWorkerPlugin::<WriteTextureWorker>::default())
    .add_plugins((
        ExtractResourcePlugin::<DisplayImage>::default(),
        ExtractResourcePlugin::<ComputeTransfer>::default(),
    ))
    .add_systems(Startup, setup_camera)
    .add_systems(
        Update,
        (handle_resize, handle_compute_params, extract_compute_view).chain(),
    );

    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.add_systems(Render, link_compute_texture.in_set(RenderSet::Prepare));
    }

    app.run();
}
