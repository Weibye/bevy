//! Illustrates how to change window settings and shows how to affect
//! the mouse pointer in various ways.

use bevy::{
    prelude::*,
    window::{Cursor, WindowTitle},
};

#[cfg(not(target_arch = "wasm32"))]
use bevy::window::{PresentMode, WindowResolution};
#[cfg(target_arch = "wasm32")]
use bevy_internal::window::WindowCanvas;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let window_settings = WindowBundle {
        title: WindowTitle::new("I am a window!"),
        resolution: WindowResolution::new(500., 300.),
        present_mode: PresentMode::AutoVsync,
        ..default()
    };

    #[cfg(target_arch = "wasm32")]
    let window_settings = WindowBundle {
        // Tells wasm to resize the window according to the available canvas
        canvas: WindowCanvas::new(None, true),
        ..default()
    };

    App::new()
        // Inserts the settings defining the first window to be spawned.
        .insert_resource(window_settings)
        .add_plugins(DefaultPlugins)
        .add_system(change_title)
        .add_system(toggle_cursor)
        .add_system(cycle_cursor_icon)
        .run();
}

/// This system will then change the title during execution
fn change_title(mut titles: Query<&mut WindowTitle, With<Window>>, time: Res<Time>) {
    for mut title in &mut titles {
        title.set(format!(
            "Seconds since startup: {}",
            time.seconds_since_startup().round()
        ));
    }
}

fn toggle_cursor(mut cursors: Query<&mut Cursor, With<Window>>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Space) {
        for mut cursor in &mut cursors {
            let visible = cursor.visible();
            let locked = cursor.locked();
            cursor.set_visible(!visible);
            cursor.set_locked(!locked);
        }
    }
}

/// This system cycles the cursor's icon through a small set of icons when clicking
fn cycle_cursor_icon(
    mut cursors: Query<&mut Cursor, With<Window>>,
    input: Res<Input<MouseButton>>,
    mut index: Local<usize>,
) {
    const ICONS: &[CursorIcon] = &[
        CursorIcon::Default,
        CursorIcon::Hand,
        CursorIcon::Wait,
        CursorIcon::Text,
        CursorIcon::Copy,
    ];

    if input.just_pressed(MouseButton::Left) {
        *index = (*index + 1) % ICONS.len();
    } else if input.just_pressed(MouseButton::Right) {
        *index = if *index == 0 {
            ICONS.len() - 1
        } else {
            *index - 1
        };
    }

    for mut cursor in &mut cursors {
        cursor.set_icon(ICONS[*index]);
    }
}
