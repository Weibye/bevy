use crate::{
    Cursor, CursorPosition, PresentMode, PrimaryWindow, Window, WindowCloseRequested, WindowClosed,
    WindowCurrentlyFocused, WindowMode, WindowPosition, WindowResizeConstraints, WindowResolution,
    WindowTitle,
};

use bevy_app::AppExit;
use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};

/// Exit the application when there are no open windows.
///
/// This system is added by the [`WindowPlugin`] in the default configuration.
/// To disable this behaviour, set `close_when_requested` (on the [`WindowPlugin`]) to `false`.
/// Ensure that you read the caveats documented on that field if doing so.
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn exit_on_all_closed(mut app_exit_events: EventWriter<AppExit>, windows: Query<&Window>) {
    if windows.iter().count() == 0 {
        app_exit_events.send(AppExit);
    }
}

/// Exit the application when the primary window has been closed
///
/// This system is added by the [`WindowPlugin`]
// TODO: More docs
pub fn exit_on_primary_closed(
    mut app_exit_events: EventWriter<AppExit>,
    primary_window: Option<Res<PrimaryWindow>>,
    mut window_close: EventReader<WindowClosed>,
) {
    if let Some(primary_window) = primary_window {
        for window in window_close.iter() {
            if primary_window.window == window.entity {
                // Primary window has been closed
                app_exit_events.send(AppExit);
            }
        }
    }
}

/// Close windows in response to [`WindowCloseRequested`] (e.g.  when the close button is pressed).
///
/// This system is added by the [`WindowPlugin`] in the default configuration.
/// To disable this behaviour, set `close_when_requested` (on the [`WindowPlugin`]) to `false`.
/// Ensure that you read the caveats documented on that field if doing so.
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn close_when_requested(mut commands: Commands, mut closed: EventReader<WindowCloseRequested>) {
    for event in closed.iter() {
        //commands.entity(entity).close();
    }
}

/// Close the focused window whenever the escape key (<kbd>Esc</kbd>) is pressed
///
/// This is useful for examples or prototyping.
pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<Entity, With<WindowCurrentlyFocused>>,
    // mut focused: Local<Option<WindowId>>,
    input: Res<Input<KeyCode>>,
) {
    // TODO: Not quite sure what this is about
    // TODO: Track this in e.g. a resource to ensure consistent behaviour across similar systems
    // for event in focused_events.iter() {
    //     *focused = event.focused.then(|| event.id);
    // }

    for focused_window in focused_windows.iter() {
        if input.just_pressed(KeyCode::Escape) {
            //commands.window(focused_window).close();
        }
    }
}

/// Add any window properties that weren't specified.
pub fn default_necessary_components(
    mut commands: Commands,
    cursor: Query<Entity, (With<Window>, Without<Cursor>)>,
    cursor_position: Query<Entity, (With<Window>, Without<CursorPosition>)>,
    present_mode: Query<Entity, (With<Window>, Without<PresentMode>)>,
    window_mode: Query<Entity, (With<Window>, Without<WindowMode>)>,
    position: Query<Entity, (With<Window>, Without<WindowPosition>)>,
    resolution: Query<Entity, (With<Window>, Without<WindowResolution>)>,
    title: Query<Entity, (With<Window>, Without<WindowTitle>)>,
    resize_constraints: Query<Entity, (With<Window>, Without<WindowResizeConstraints>)>,
) {
    for window in &cursor {
        commands.entity(window).insert(Cursor::default());
    }

    for window in &cursor_position {
        commands.entity(window).insert(CursorPosition::default());
    }

    for window in &present_mode {
        commands.entity(window).insert(PresentMode::default());
    }

    for window in &window_mode {
        commands.entity(window).insert(WindowMode::default());
    }

    for window in &position {
        commands.entity(window).insert(WindowPosition::default());
    }

    for window in &resolution {
        commands.entity(window).insert(WindowResolution::default());
    }

    for window in &title {
        commands.entity(window).insert(WindowTitle::default());
    }

    for window in &resize_constraints {
        commands
            .entity(window)
            .insert(WindowResizeConstraints::default());
    }
}
