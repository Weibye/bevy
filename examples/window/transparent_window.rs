//! Shows how to display a window in transparent mode.
//!
//! This feature works as expected depending on the platform. Please check the
//! [documentation](https://docs.rs/bevy/latest/bevy/prelude/struct.WindowDescriptor.html#structfield.transparent)
//! for more details.

use bevy::{
    prelude::*,
    window::{WindowBundle, WindowDecorations, WindowTransparency},
};

fn main() {
    App::new()
        // ClearColor must have 0 alpha, otherwise some color will bleed through
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(WindowBundle {
            // Setting `transparent` allows the `ClearColor`'s alpha value to take effect
            transparency: WindowTransparency::Transparent,
            // Disabling window decorations to make it feel more like a widget than a window
            decorations: WindowDecorations::Undecorated,
            ..default()
        })
        .add_startup_system(setup)
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("branding/icon.png"),
        ..default()
    });
}
