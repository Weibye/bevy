use bevy_ecs::{
    entity::{Entities, Entity},
    event::EventWriter,
    prelude::{Added, Changed, With},
    system::{Commands, NonSendMut, Query, RemovedComponents, Res},
};
use bevy_utils::tracing::{error, info};
use bevy_window::{
    Cursor, CursorPosition, PrimaryWindow, Window, WindowClosed, WindowComponents, WindowHandle,
    WindowMode, WindowPosition, WindowResizeConstraints, WindowResolution, WindowState,
    WindowTitle,
};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event_loop::EventLoopWindowTarget,
};

#[cfg(target_arch = "wasm32")]
use crate::web_resize::{CanvasParentResizeEventChannel, WINIT_CANVAS_SELECTOR};
use crate::{converters, get_best_videomode, get_fitting_videomode, WinitWindows};
#[cfg(target_arch = "wasm32")]
use bevy_ecs::system::ResMut;

/// System responsible for creating new windows whenever a `Window` component is added
/// to an entity.
///
/// This will default any necessary components if they are not already added.
pub(crate) fn create_window(
    mut commands: Commands,
    event_loop: &EventLoopWindowTarget<()>,
    created_windows: Query<(Entity, WindowComponents), Added<Window>>,
    mut winit_windows: NonSendMut<WinitWindows>,
    #[cfg(target_arch = "wasm32")] mut event_channel: ResMut<CanvasParentResizeEventChannel>,
) {
    for (window_entity, components) in &created_windows {
        if let Some(_) = winit_windows.get_window(window_entity) {
            // Just a safe guard
            continue;
        }

        info!("Creating a new window: {:?}", window_entity);

        let winit_window = winit_windows.create_window(&event_loop, window_entity, &components);

        commands
            .entity(window_entity)
            .insert(WindowHandle::new(winit_window.raw_window_handle()));

        #[cfg(target_arch = "wasm32")]
        {
            if components.canvas.fit_canvas_to_parent() {
                let selector = if let Some(selector) = &components.canvas.canvas() {
                    selector
                } else {
                    WINIT_CANVAS_SELECTOR
                };
                event_channel.listen_to_selector(window_entity, selector);
            }
        }
    }
}

pub(crate) fn despawn_window(
    mut commands: Commands,
    entities: &Entities,
    primary: Option<Res<PrimaryWindow>>,
    closed: RemovedComponents<Window>,
    mut close_events: EventWriter<WindowClosed>,
    mut winit_windows: NonSendMut<WinitWindows>,
) {
    for window in closed.iter() {
        winit_windows.remove_window(window);

        if entities.contains(window) {
            commands.entity(window).despawn();
        }

        if let Some(ref primary) = primary {
            if primary.window == window {
                commands.remove_resource::<PrimaryWindow>();
            }
        }

        close_events.send(WindowClosed { window });
    }
}

pub(crate) fn update_title(
    changed_windows: Query<(Entity, &WindowTitle), (With<Window>, Changed<WindowTitle>)>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, title) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            winit_window.set_title(title.as_str());
        }
    }
}

pub(crate) fn update_window_mode(
    changed_windows: Query<
        (Entity, &WindowMode, &WindowResolution),
        (With<Window>, Changed<WindowMode>),
    >,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, mode, resolution) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            match mode {
                bevy_window::WindowMode::BorderlessFullscreen => {
                    winit_window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                }
                bevy_window::WindowMode::Fullscreen => {
                    winit_window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(
                        get_best_videomode(&winit_window.current_monitor().unwrap()),
                    )));
                }
                bevy_window::WindowMode::SizedFullscreen => winit_window.set_fullscreen(Some(
                    winit::window::Fullscreen::Exclusive(get_fitting_videomode(
                        &winit_window.current_monitor().unwrap(),
                        resolution.width() as u32,
                        resolution.height() as u32,
                    )),
                )),
                bevy_window::WindowMode::Windowed => winit_window.set_fullscreen(None),
            }
        }
    }
}

pub(crate) fn update_resolution(
    changed_windows: Query<(Entity, &WindowResolution), (With<Window>, Changed<WindowResolution>)>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, resolution) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            let physical_size = LogicalSize::new(resolution.width(), resolution.height())
                .to_physical::<f64>(resolution.scale_factor());
            winit_window.set_inner_size(physical_size);
        }
    }
}

pub(crate) fn update_cursor_position(
    changed_windows: Query<(Entity, &CursorPosition), (With<Window>, Changed<CursorPosition>)>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, cursor_position) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            if let Some(physical_position) = cursor_position.physical_position() {
                let inner_size = winit_window.inner_size();

                let position = PhysicalPosition::new(
                    physical_position.x,
                    // Flip the coordinate space back to winit's context.
                    inner_size.height as f64 - physical_position.y,
                );

                if let Err(err) = winit_window.set_cursor_position(position) {
                    error!("could not set cursor position: {:?}", err);
                }
            }
        }
    }
}

pub(crate) fn update_cursor(
    changed_windows: Query<(Entity, &Cursor), (With<Window>, Changed<Cursor>)>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, cursor) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            winit_window.set_cursor_icon(converters::convert_cursor_icon(cursor.icon()));

            if let Err(err) = winit_window.set_cursor_grab(cursor.locked()) {
                let err_desc = if cursor.locked() { "grab" } else { "ungrab" };
                error!("Unable to {} cursor: {}", err_desc, err);
            }

            winit_window.set_cursor_visible(cursor.visible());
        }
    }
}

pub(crate) fn update_resize_constraints(
    changed_windows: Query<
        (Entity, &WindowResizeConstraints),
        (With<Window>, Changed<WindowResizeConstraints>),
    >,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, resize_constraints) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            let constraints = resize_constraints.check_constraints();
            let min_inner_size = LogicalSize {
                width: constraints.min_width,
                height: constraints.min_height,
            };
            let max_inner_size = LogicalSize {
                width: constraints.max_width,
                height: constraints.max_height,
            };

            winit_window.set_min_inner_size(Some(min_inner_size));
            if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                winit_window.set_max_inner_size(Some(max_inner_size));
            }
        }
    }
}

pub(crate) fn update_window_position(
    changed_windows: Query<
        (Entity, &WindowPosition, &WindowResolution),
        (With<Window>, Changed<WindowPosition>),
    >,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, position, resolution) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            if let Some(position) = crate::winit_window_position(
                position,
                resolution,
                winit_window.available_monitors(),
                winit_window.primary_monitor(),
                winit_window.current_monitor(),
            ) {
                winit_window.set_outer_position(position);
            }
        }
    }
}

pub(crate) fn update_window_state(
    changed_windows: Query<(Entity, &WindowState), (With<Window>, Changed<WindowState>)>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, state) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            match state {
                WindowState::Normal => {
                    winit_window.set_minimized(false);
                    winit_window.set_maximized(false);
                }
                WindowState::Maximized => {
                    winit_window.set_maximized(true);
                }
                WindowState::Minimized => {
                    winit_window.set_minimized(true);
                }
            }
        }
    }
}
