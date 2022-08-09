use crate::{PrimaryWindow, Window, WindowCloseRequested, WindowClosed, WindowFocus};

use bevy_app::AppExit;
use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};
use bevy_utils::tracing::warn;

/// Exit the application when there are no open windows.
///
/// This system is added by the [`WindowPlugin`] in the default configuration.
/// To disable this behaviour, set `close_when_requested` (on the [`WindowPlugin`]) to `false`.
/// Ensure that you read the caveats documented on that field if doing so.
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn exit_on_all_closed(mut app_exit_events: EventWriter<AppExit>, windows: Query<&Window>) {
    if windows.iter().count() == 0 {
        println!("no windows are open, exiting");
        app_exit_events.send(AppExit);
    }
}

/// Exit the application when the primary window has been closed
///
/// This system is added by the [`WindowPlugin`]
// TODO: More docs
pub fn exit_on_primary_closed(
    mut app_exit_events: EventWriter<AppExit>,
    primary_window: Res<PrimaryWindow>,
    mut window_close: EventReader<WindowClosed>,
) {
    for window in window_close.iter() {
        warn!(
            "primary_window: {:?}, closed: {:?}",
            primary_window.window, window.entity
        );
        if primary_window.window == window.entity {
            // Primary window has been closed
            app_exit_events.send(AppExit);
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
pub fn close_when_requested(
    mut closed: EventReader<WindowCloseRequested>,
    mut window_closed: EventWriter<WindowClosed>,
) {
    for event in closed.iter() {
        window_closed.send(WindowClosed {
            entity: event.entity,
        })
    }
}

/// Close the focused window whenever the escape key (<kbd>Esc</kbd>) is pressed
///
/// This is useful for examples or prototyping.
pub fn close_on_esc(
    focused_windows: Query<(Entity, &WindowFocus)>,
    mut window_closed: EventWriter<WindowClosed>,
    input: Res<Input<KeyCode>>,
) {
    for (entity, focus) in focused_windows.iter() {
        if !focus.focused() {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            window_closed.send(WindowClosed { entity });
        }
    }
}
