//! Uses two windows to visualize a 3D model from different angles.

use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{PresentMode, PrimaryWindow, Window, WindowResolution, WindowState, WindowTitle},
};

fn main() {
    App::new()
        // Primary window gets spawned as a result of `DefaultPlugins`
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_scene)
        .add_startup_system(setup_second_window)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    primary_window: Res<PrimaryWindow>,
) {
    // add entities to the world
    commands.spawn_bundle(SceneBundle {
        scene: asset_server.load("models/monkey/Monkey.gltf#Scene0"),
        ..default()
    });
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 5.0, 4.0),
        ..default()
    });
    // main camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            target: RenderTarget::Window(primary_window.window),
            ..default()
        },
        ..default()
    });
}

fn setup_second_window(mut commands: Commands) {
    // Spawn a new entity that will act as our window id
    let window_id = commands
        .spawn()
        .insert_bundle(WindowBundle {
            title: WindowTitle::new("Second window"),
            state: WindowState::Minimized,
            ..Default::default()
        })
        .id();

    // second window camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(6.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            target: RenderTarget::Window(window_id),
            ..default()
        },
        ..default()
    });

    let window_id = commands
        .spawn()
        .insert_bundle(WindowBundle {
            title: WindowTitle::new("Third window"),
            state: WindowState::Maximized,
            ..Default::default()
        })
        .id();

    // third window camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-6.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            target: RenderTarget::Window(window_id),
            ..default()
        },
        ..default()
    });
}
