mod compute;
mod config;
mod render;
mod voxel_map;

use crate::compute::{WriteTextureWorker, handle_compute_params};
use crate::config::{AppSettings, Brick, Material};
use crate::render::*;
use crate::voxel_map::{Sector, SvoStorage, VoxelWorld};
use bevy::input::mouse::MouseMotion;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::{Render, RenderApp, RenderSet, extract_resource::ExtractResourcePlugin};
use bevy::window::{CursorGrabMode, PresentMode, WindowResolution};
use bevy_app_compute::prelude::*;
use iyes_perf_ui::PerfUiPlugin;

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
                    present_mode : PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes_override: Some(true),
                ..default()
            }),
    )
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
    .insert_resource(settings)
    .add_plugins(AppComputePlugin)
    .add_plugins(AppComputeWorkerPlugin::<WriteTextureWorker>::default())
    .add_plugins((
        ExtractResourcePlugin::<DisplayImage>::default(),
        ExtractResourcePlugin::<ComputeTransfer>::default(),
    ))
    .add_systems(Startup, (setup, spawn_sphere))
    .add_systems(
        Update,
        (
            camera_movement_system,
            handle_resize,
            rebuild_svo,
            upload_to_gpu,
            handle_compute_params,
            extract_compute_view,
        )
            .chain(),
    );

    app.insert_resource(VoxelWorld {
        palette: vec![
            Material::default(),
            Material {
                color: [1.0, 0.0, 0.0],
                ..default()
            },
        ],
        ..default()
    })
    .insert_resource(SvoStorage {
        tree_scale: 6,
        ..default()
    });

    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.add_systems(Render, link_compute_texture.in_set(RenderSet::Prepare));
    }

    app.run();
}

pub fn spawn_sphere(mut world: ResMut<VoxelWorld>) {
    let radius = 32;
    let center = IVec3::splat(32);
    for x in -radius..=radius {
        for y in -radius..=radius {
            for z in -radius..=radius {
                let offset = IVec3::new(x, y, z);
                if offset.length_squared() <= (radius * radius) {
                    let pos = center + offset;
                    let sector_pos = pos >> 6;
                    let sector = world.sectors.entry(sector_pos).or_insert(Sector {
                        bricks: HashMap::default(),
                    });

                    let local_pos_in_sector = pos - (sector_pos << 6);
                    let brick_pos: IVec3 = local_pos_in_sector >> 2;
                    let brick_idx = (brick_pos.x + brick_pos.y * 16 + brick_pos.z * 256) as u32;

                    let brick = sector
                        .bricks
                        .entry(brick_idx)
                        .or_insert(Brick { voxels: [0; 64] });

                    let v_local: IVec3 = local_pos_in_sector & 3;
                    let v_idx = (v_local.x + v_local.z * 4 + v_local.y * 16) as usize;
                    brick.voxels[v_idx] = 1;
                }
            }
        }
    }
    println!("Sphere generated!");
}

pub fn camera_movement_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera_q: Query<&mut Transform, With<VoxelCamera>>,
) {
    let Ok(mut transform) = camera_q.single_mut() else {
        return;
    };

    let mut rotation_move = Vec2::ZERO;
    for event in mouse_motion.read() {
        rotation_move += event.delta;
    }

    if rotation_move.length_squared() > 0.0 {
        let sensitivity = 0.002;
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

        yaw -= rotation_move.x * sensitivity;
        pitch += rotation_move.y * sensitivity;
        pitch = pitch.clamp(-1.5, 1.5);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }

    let mut velocity = Vec3::ZERO;
    let local_z = transform.forward();
    let local_x = transform.right();

    if keyboard.pressed(KeyCode::KeyW) {
        velocity += *local_z;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        velocity -= *local_z;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        velocity += *local_x;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        velocity -= *local_x;
    }

    let speed = 100.0;
    transform.translation += velocity.normalize_or_zero() * speed * time.delta_secs();
}
fn lock_cursor(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut().unwrap();
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
    window.cursor_options.visible = false;
}

fn rebuild_svo(world: Res<VoxelWorld>, mut svo: ResMut<SvoStorage>) {
    if (world.is_changed()) {
        world.generate_svo(&mut svo);
        println!("Generated SVO");
    }
}

fn upload_to_gpu(
    svo: Res<SvoStorage>,
    mut worker: ResMut<AppComputeWorker<WriteTextureWorker>>,
    display_image: Res<DisplayImage>,
) {
    if svo.is_changed() || display_image.is_changed() {
        worker.write_slice("nodePool", &svo.nodes);
        worker.write_slice("leafData", &svo.leaf_data);
        println!("Uploaded NodePool");
    }
}
