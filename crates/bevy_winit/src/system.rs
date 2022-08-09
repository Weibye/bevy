use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    prelude::{Added, Changed, With},
    system::{Commands, Local, NonSendMut, Query, Res},
};
use bevy_utils::tracing::{error, info};
use bevy_window::{
    Cursor, CursorPosition, PrimaryWindow, Window, WindowClosed, WindowComponents, WindowHandle,
    WindowMode, WindowPosition, WindowResizeConstraints, WindowResolution, WindowTitle,
};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition},
    event_loop::EventLoopWindowTarget,
};

use crate::{converters, get_best_videomode, get_fitting_videomode, WinitWindows};

/// System responsible for creating new windows whenever a `Window` component is added
/// to an entity.
///
/// This will default any necessary components if they are not already added.
pub fn create_window_system(
    mut commands: Commands,
    event_loop: &EventLoopWindowTarget<()>,
    created_windows: Query<(Entity, WindowComponents), Added<Window>>,
    mut winit_windows: NonSendMut<WinitWindows>,
) {
    for (window_entity, components) in &created_windows {
        if let Some(_) = winit_windows.get_window(window_entity) {
            // Just a safe guard
            continue;
        }

        info!("Creating a new window: {:?}", window_entity);

        let winit_window = winit_windows.create_window(&event_loop, window_entity, &components);
        info!("winit window: {:?}", window_entity);

        commands
            .entity(window_entity)
            .insert(WindowHandle::new(winit_window.raw_window_handle()));

        // TODO: Fix this
        #[cfg(target_arch = "wasm32")]
        {
            let channel = world.resource_mut::<web_resize::CanvasParentResizeEventChannel>();
            if create_window_event.descriptor.fit_canvas_to_parent {
                let selector = if let Some(selector) = &create_window_event.descriptor.canvas {
                    selector
                } else {
                    web_resize::WINIT_CANVAS_SELECTOR
                };
                channel.listen_to_selector(create_window_event.entity, selector);
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct RemoveWindowBuffer {
    remove: Vec<Entity>,
}

pub fn window_destroyed(
    mut commands: Commands,
    primary: Res<PrimaryWindow>,
    mut closed: EventReader<WindowClosed>,
    mut winit_windows: NonSendMut<WinitWindows>,
    mut buffer: Local<RemoveWindowBuffer>,
) {
    for entity in buffer.remove.drain(..) {
        winit_windows.remove_window(entity);

        commands.entity(entity).despawn();
        if primary.window == entity {
            commands.remove_resource::<PrimaryWindow>();
        }
    }

    // We buffer for a frame so that other systems clean up first
    // regardless of ordering.
    for event in closed.iter() {
        buffer.remove.push(event.entity);
    }
}

pub fn update_title(
    changed_windows: Query<(Entity, &WindowTitle), (With<Window>, Changed<WindowTitle>)>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, title) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            winit_window.set_title(title.as_str());
        }
    }
}

pub fn update_window_mode(
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

pub fn update_resolution(
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

pub fn update_cursor_position(
    changed_windows: Query<(Entity, &CursorPosition), (With<Window>, Changed<CursorPosition>)>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, cursor_position) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            if let Some(position) = cursor_position.position() {
                let inner_size = winit_window
                    .inner_size()
                    .to_logical::<f64>(winit_window.scale_factor());

                let position = LogicalPosition::new(position.x, inner_size.height - position.y);
                if let Err(err) = winit_window.set_cursor_position(position) {
                    error!("could not set cursor position: {:?}", err);
                }
            }
        }
    }
}

pub fn update_cursor(
    changed_windows: Query<(Entity, &Cursor), (With<Window>, Changed<Cursor>)>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, cursor) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            winit_window.set_cursor_icon(converters::convert_cursor_icon(cursor.icon()));

            winit_window
                .set_cursor_grab(cursor.locked())
                .unwrap_or_else(|e| error!("Unable to un/grab cursor: {}", e));

            winit_window.set_cursor_visible(cursor.visible());
        }
    }
}

pub fn update_resize_constraints(
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

pub fn update_window_position(
    changed_windows: Query<(Entity, &WindowPosition), (With<Window>, Changed<WindowPosition>)>,
    winit_windows: NonSendMut<WinitWindows>,
) {
    for (entity, position) in changed_windows.iter() {
        if let Some(winit_window) = winit_windows.get_window(entity) {
            match position {
                WindowPosition::At(position) => {
                    winit_window.set_outer_position(PhysicalPosition {
                        x: position[0],
                        y: position[1],
                    });
                }
                WindowPosition::Automatic => {}
                WindowPosition::Centered(monitor) => {
                    // TODO: figure out what to do here if anything
                }
            }
        }
    }
}
