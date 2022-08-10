//! Demonstrates how to grab and hide the mouse cursor.

use bevy::{
    prelude::*,
    window::{Cursor, PrimaryWindow},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_system(grab_mouse)
        .run();
}

// This system grabs the mouse when the left mouse button is pressed
// and releases it when the escape key is pressed
fn grab_mouse(
    primary_window: Res<PrimaryWindow>,
    mut cursors: Query<&mut Cursor, With<Window>>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut cursor = cursors
        .get_mut(primary_window.window)
        .expect("Expected cursor for primary window");

    if mouse.just_pressed(MouseButton::Left) {
        cursor.set_visible(false);
        cursor.set_locked(true);
    }

    if key.just_pressed(KeyCode::Escape) {
        cursor.set_visible(true);
        cursor.set_locked(false);
    }
}
