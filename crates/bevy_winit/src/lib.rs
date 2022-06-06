mod converters;
mod system;
#[cfg(target_arch = "wasm32")]
mod web_resize;
mod winit_config;
mod winit_windows;

use core::panic;

use bevy_ecs::system::Command;
use system::{
    destroy_windows, update_cursor_icon, update_cursor_lock_mode, update_cursor_position,
    update_cursor_visibility, update_decorations, update_maximized, update_minimized,
    update_position, update_present_mode, update_resizable, update_resize_contraints,
    update_resolution, update_scale_factor, update_title, update_window_mode, window_destroyed,
};
use winit::window;
pub use winit_config::*;
pub use winit_windows::*;

use bevy_app::{App, AppExit, CoreStage, Plugin};
use bevy_ecs::prelude::*;
use bevy_ecs::{
    event::{Events, ManualEventReader},
    world::World,
};
use bevy_input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel},
    touch::TouchInput,
};
use bevy_math::{ivec2, DVec2, UVec2, Vec2};
use bevy_utils::{
    tracing::{error, info, trace, warn},
    Instant,
};
use bevy_window::{
    CreateWindow, CursorEntered, CursorLeft, CursorMoved, FileDragAndDrop, ModifiesWindows,
    PrimaryWindow, ReceivedCharacter, RequestRedraw, Window, WindowBackendScaleFactorChanged,
    WindowCloseRequested, WindowCreated, WindowCurrentlyFocused, WindowCursorPosition,
    WindowFocused, WindowMoved, WindowPosition, WindowResized, WindowResolution,
    WindowScaleFactorChanged,
};

use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event::{self, DeviceEvent, Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
};

#[derive(Default)]
pub struct WinitPlugin;

impl Plugin for WinitPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<WinitWindows>()
            .init_resource::<WinitSettings>()
            .set_runner(winit_runner)
            // TODO: Verify that this actually works and does not cause any race-conditions or strange ordering issues
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .label(ModifiesWindows)
                    .with_system(update_title)
                    .with_system(update_window_mode)
                    .with_system(update_decorations)
                    .with_system(update_scale_factor)
                    .with_system(update_resizable)
                    .with_system(update_position)
                    .with_system(update_minimized)
                    .with_system(update_maximized)
                    .with_system(update_resolution)
                    .with_system(update_cursor_icon)
                    .with_system(update_cursor_lock_mode)
                    .with_system(update_cursor_visibility)
                    .with_system(update_cursor_position)
                    .with_system(update_resize_contraints)
                    .with_system(update_present_mode)
                    .with_system(destroy_windows), // TODO: This should probably go last?
                                                   // .with_system(window_destroyed) // TODO: Unsure if this is the correct approach
            );
        #[cfg(target_arch = "wasm32")]
        app.add_plugin(web_resize::CanvasParentResizePlugin);
        let event_loop = EventLoop::new();
        #[cfg(not(target_os = "android"))]
        let mut create_window_reader = WinitCreateWindowReader::default();
        // TODO: Test if any issues has been caused here
        // Note that we create a window here "early" because WASM/WebGL requires the window to exist prior to initializing
        // the renderer.
        #[cfg(not(target_os = "android"))]
        handle_create_window_events(&mut app.world, &event_loop, &mut create_window_reader.0);
        app.insert_resource(create_window_reader)
            .insert_non_send_resource(event_loop);
    }
}

fn run<F>(event_loop: EventLoop<()>, event_handler: F) -> !
where
    F: 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    event_loop.run(event_handler)
}

// TODO: It may be worth moving this cfg into a procedural macro so that it can be referenced by
// a single name instead of being copied around.
// https://gist.github.com/jakerr/231dee4a138f7a5f25148ea8f39b382e seems to work.
#[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn run_return<F>(event_loop: &mut EventLoop<()>, event_handler: F)
where
    F: FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    use winit::platform::run_return::EventLoopExtRunReturn;
    event_loop.run_return(event_handler);
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
fn run_return<F>(_event_loop: &mut EventLoop<()>, _event_handler: F)
where
    F: FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    panic!("Run return is not supported on this platform!")
}

pub fn winit_runner(app: App) {
    winit_runner_with(app);
}

// #[cfg(any(
//     target_os = "linux",
//     target_os = "dragonfly",
//     target_os = "freebsd",
//     target_os = "netbsd",
//     target_os = "openbsd"
// ))]
// pub fn winit_runner_any_thread(app: App) {
//     winit_runner_with(app, EventLoop::new_any_thread());
// }

/// Stores state that must persist between frames.
struct WinitPersistentState {
    /// Tracks whether or not the application is active or suspended.
    active: bool,
    /// Tracks whether or not an event has occurred this frame that would trigger an update in low
    /// power mode. Should be reset at the end of every frame.
    low_power_event: bool,
    /// Tracks whether the event loop was started this frame because of a redraw request.
    redraw_request_sent: bool,
    /// Tracks if the event loop was started this frame because of a `WaitUntil` timeout.
    timeout_reached: bool,
    last_update: Instant,
}
impl Default for WinitPersistentState {
    fn default() -> Self {
        Self {
            active: true,
            low_power_event: false,
            redraw_request_sent: false,
            timeout_reached: false,
            last_update: Instant::now(),
        }
    }
}

#[derive(Default, Resource)]
struct WinitCreateWindowReader(ManualEventReader<CreateWindow>);

// TODO: Refactor this to work with new pattern
pub fn winit_runner_with(mut app: App) {
    // TODO: Understand what removing and adding this does
    let mut event_loop = app
        .world
        .remove_non_send_resource::<EventLoop<()>>()
        .unwrap();
    let mut create_window_event_reader = app
        .world
        .remove_resource::<WinitCreateWindowReader>()
        .unwrap()
        .0;
    let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();
    let mut redraw_event_reader = ManualEventReader::<RequestRedraw>::default();
    let mut winit_state = WinitPersistentState::default();
    app.world
        .insert_non_send_resource(event_loop.create_proxy());

    let return_from_run = app.world.resource::<WinitSettings>().return_from_run;

    trace!("Entering winit event loop");

    let event_handler = move |event: Event<()>,
                              event_loop: &EventLoopWindowTarget<()>,
                              control_flow: &mut ControlFlow| {
        match event {
            event::Event::NewEvents(start) => {
                let winit_config = app.world.resource::<WinitSettings>();

                // Collection of windows
                let mut windows_query = app.world.query_filtered::<Entity, With<Window>>();
                let windows: Vec<Entity> = windows_query.iter(&app.world).collect();

                // True if _any_ windows are currently being focused
                let mut windows_focused_query = app
                    .world
                    .query_filtered::<Entity, (With<Window>, With<WindowCurrentlyFocused>)>();
                let focused = windows_focused_query
                    .iter(&app.world)
                    .collect::<Vec<Entity>>()
                    .len()
                    > 0;

                // Check if either the `WaitUntil` timeout was triggered by winit, or that same
                // amount of time has elapsed since the last app update. This manual check is needed
                // because we don't know if the criteria for an app update were met until the end of
                // the frame.
                let auto_timeout_reached = matches!(start, StartCause::ResumeTimeReached { .. });
                let now = Instant::now();
                let manual_timeout_reached = match winit_config.update_mode(focused) {
                    UpdateMode::Continuous => false,
                    UpdateMode::Reactive { max_wait }
                    | UpdateMode::ReactiveLowPower { max_wait } => {
                        now.duration_since(winit_state.last_update) >= *max_wait
                    }
                };
                // The low_power_event state and timeout must be reset at the start of every frame.
                winit_state.low_power_event = false;
                winit_state.timeout_reached = auto_timeout_reached || manual_timeout_reached;
            }
            event::Event::WindowEvent {
                event,
                window_id: winit_window_id,
                ..
            } => {
                // TODO: Should the queries happen on the world cell from this point onwards instead of app.world?
                let world = app.world.cell();
                let winit_windows = world.non_send_resource_mut::<WinitWindows>();

                // Query windows from world
                let mut windows_query = app.world.query_filtered::<Entity, With<Window>>();
                let mut windows: Vec<Entity> = windows_query.iter(&app.world).collect();

                let window_entity =
                    if let Some(entity) = winit_windows.get_window_entity(winit_window_id) {
                        entity
                    } else {
                        // TODO: This seems like it can cause problems now
                        warn!(
                            "Skipped event for unknown winit Window Id {:?}",
                            winit_window_id
                        );
                        return;
                    };

                // TODO: Is there an edge-case introduced by removing this?
                // let window = if let Some(window) = windows.get_mut(window_entity) {
                //     window
                // } else {
                //     // If we're here, this window was previously opened
                //     info!("Skipped event for closed window: {:?}", window_entity);
                //     return;
                // };
                winit_state.low_power_event = true;

                match event {
                    WindowEvent::Resized(size) => {
                        // TODO:
                        if let Some(mut resolution_component) =
                            app.world.get_mut::<WindowResolution>(window_entity)
                        {
                            // Update component
                            resolution_component
                                .update_actual_size_from_backend(size.width, size.height);

                            // Send event to notify change
                            let mut resize_events = world.resource_mut::<Events<WindowResized>>();
                            resize_events.send(WindowResized {
                                entity: window_entity,
                                width: resolution_component.width(),
                                height: resolution_component.height(),
                            });
                        } else {
                            // TODO: Helpful panic comment
                            panic!("Window does not have a valid WindowResolution component");
                        }
                    }
                    WindowEvent::CloseRequested => {
                        let mut window_close_requested_events =
                            world.resource_mut::<Events<WindowCloseRequested>>();
                        window_close_requested_events.send(WindowCloseRequested {
                            entity: window_entity,
                        });
                    }
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        let mut keyboard_input_events =
                            world.resource_mut::<Events<KeyboardInput>>();
                        keyboard_input_events.send(converters::convert_keyboard_input(input));
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let mut cursor_moved_events = world.resource_mut::<Events<CursorMoved>>();
                        let winit_window = winit_windows.get_window(window_entity).unwrap();
                        let inner_size = winit_window.inner_size();

                        // Components
                        // Need WindowResolution component
                        let window_resolution = app
                            .world
                            .get::<WindowResolution>(window_entity)
                            .expect("Window should have a WindowResolution component");

                        // Need cursorposition component
                        let mut cursor_position = app
                            .world
                            .get_mut::<WindowCursorPosition>(window_entity)
                            .expect("Window should have a WindowCursorPosition component");

                        // TODO: Why is this necessary? Improve comment as to why
                        // move origin to bottom left
                        let y_position = inner_size.height as f64 - position.y;

                        let physical_position = DVec2::new(position.x, y_position);

                        cursor_position.update_position_from_backend(Some(physical_position));

                        // Event
                        cursor_moved_events.send(CursorMoved {
                            entity: window_entity,
                            position: (physical_position / window_resolution.scale_factor())
                                .as_vec2(),
                        });
                    }
                    WindowEvent::CursorEntered { .. } => {
                        let mut cursor_entered_events =
                            world.resource_mut::<Events<CursorEntered>>();
                        cursor_entered_events.send(CursorEntered {
                            entity: window_entity,
                        });
                    }
                    WindowEvent::CursorLeft { .. } => {
                        // Component
                        let mut cursor_position = app
                            .world
                            .get_mut::<WindowCursorPosition>(window_entity)
                            .expect("Window should have a WindowCursorComponent component");
                        cursor_position.update_position_from_backend(None);

                        // Event
                        let mut cursor_left_events = world.resource_mut::<Events<CursorLeft>>();
                        cursor_left_events.send(CursorLeft {
                            entity: window_entity,
                        });
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let mut mouse_button_input_events =
                            world.resource_mut::<Events<MouseButtonInput>>();
                        mouse_button_input_events.send(MouseButtonInput {
                            button: converters::convert_mouse_button(button),
                            state: converters::convert_element_state(state),
                        });
                    }
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        event::MouseScrollDelta::LineDelta(x, y) => {
                            let mut mouse_wheel_input_events =
                                world.resource_mut::<Events<MouseWheel>>();
                            mouse_wheel_input_events.send(MouseWheel {
                                unit: MouseScrollUnit::Line,
                                x,
                                y,
                            });
                        }
                        event::MouseScrollDelta::PixelDelta(p) => {
                            let mut mouse_wheel_input_events =
                                world.resource_mut::<Events<MouseWheel>>();
                            mouse_wheel_input_events.send(MouseWheel {
                                unit: MouseScrollUnit::Pixel,
                                x: p.x as f32,
                                y: p.y as f32,
                            });
                        }
                    },
                    WindowEvent::Touch(touch) => {
                        let mut touch_input_events = world.resource_mut::<Events<TouchInput>>();

                        let window_resolution = app
                            .world
                            .get::<WindowResolution>(window_entity)
                            .expect("Window should have a WindowResolution component");

                        let mut location =
                            touch.location.to_logical(window_resolution.scale_factor());

                        // On a mobile window, the start is from the top while on PC/Linux/OSX from
                        // bottom
                        if cfg!(target_os = "android") || cfg!(target_os = "ios") {
                            // Get windows_resolution of the entity currently set as primary window
                            let primary_window = world.resource::<PrimaryWindow>().window.expect(
                                "There should be a primary window but it seems that it is not",
                            ); // TODO: Update panic comment
                            let primary_window_resolution =
                                app.world.get::<WindowResolution>(primary_window).expect(
                                    "Primary window should have a valid WindowResolution component",
                                );
                            location.y = primary_window_resolution.height() - location.y;
                        }
                        touch_input_events.send(converters::convert_touch_input(touch, location));
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        let mut char_input_events =
                            world.resource_mut::<Events<ReceivedCharacter>>();

                        char_input_events.send(ReceivedCharacter {
                            entity: window_entity,
                            char: c,
                        });
                    }
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size,
                    } => {
                        let mut backend_scale_factor_change_events =
                            world.resource_mut::<Events<WindowBackendScaleFactorChanged>>();
                        backend_scale_factor_change_events.send(WindowBackendScaleFactorChanged {
                            entity: window_entity,
                            scale_factor,
                        });

                        // Components
                        let mut window_resolution = app
                            .world
                            .get_mut::<WindowResolution>(window_entity)
                            .expect("Window should have a WindowResolution component");

                        let prior_factor = window_resolution.scale_factor();
                        window_resolution.update_scale_factor_from_backend(scale_factor);
                        let new_factor = window_resolution.scale_factor();

                        if let Some(forced_factor) = window_resolution.scale_factor_override() {
                            // If there is a scale factor override, then force that to be used
                            // Otherwise, use the OS suggested size
                            // We have already told the OS about our resize constraints, so
                            // the new_inner_size should take those into account
                            *new_inner_size = winit::dpi::LogicalSize::new(
                                window_resolution.requested_width(),
                                window_resolution.requested_height(),
                            )
                            .to_physical::<u32>(forced_factor);
                            // TODO: Should this not trigger a WindowsScaleFactorChanged?
                        } else if approx::relative_ne!(new_factor, prior_factor) {
                            // TODO: Trigger event with new scale_factor if they are approximately the same?
                            // Is this correct? Should it not be reversed?
                            let mut scale_factor_change_events =
                                world.resource_mut::<Events<WindowScaleFactorChanged>>();

                            scale_factor_change_events.send(WindowScaleFactorChanged {
                                entity: window_entity,
                                scale_factor,
                            });
                        }

                        let new_logical_width = new_inner_size.width as f64 / new_factor;
                        let new_logical_height = new_inner_size.height as f64 / new_factor;
                        if approx::relative_ne!(window_resolution.width() as f64, new_logical_width)
                            || approx::relative_ne!(
                                window_resolution.height() as f64,
                                new_logical_height
                            )
                        {
                            let mut resize_events = world.resource_mut::<Events<WindowResized>>();
                            resize_events.send(WindowResized {
                                entity: window_entity,
                                width: new_logical_width as f32,
                                height: new_logical_height as f32,
                            });
                        }
                        window_resolution.update_actual_size_from_backend(
                            new_inner_size.width,
                            new_inner_size.height,
                        );
                    }
                    WindowEvent::Focused(focused) => {
                        // Component
                        let mut entity_mut = app
                            .world
                            .get_entity_mut(window_entity)
                            .expect("Entity for window should exist");

                        if focused {
                            entity_mut.insert(WindowCurrentlyFocused);
                        } else {
                            entity_mut.remove::<WindowCurrentlyFocused>();
                        }

                        // Event
                        let mut focused_events = world.resource_mut::<Events<WindowFocused>>();
                        focused_events.send(WindowFocused {
                            entity: window_entity,
                            focused,
                        });
                    }
                    WindowEvent::DroppedFile(path_buf) => {
                        let mut events = world.resource_mut::<Events<FileDragAndDrop>>();
                        events.send(FileDragAndDrop::DroppedFile {
                            entity: window_entity,
                            path_buf,
                        });
                    }
                    WindowEvent::HoveredFile(path_buf) => {
                        let mut events = world.resource_mut::<Events<FileDragAndDrop>>();
                        events.send(FileDragAndDrop::HoveredFile {
                            entity: window_entity,
                            path_buf,
                        });
                    }
                    WindowEvent::HoveredFileCancelled => {
                        let mut events = world.resource_mut::<Events<FileDragAndDrop>>();
                        events.send(FileDragAndDrop::HoveredFileCancelled {
                            entity: window_entity,
                        });
                    }
                    WindowEvent::Moved(position) => {
                        let position = ivec2(position.x, position.y);
                        // Component
                        let mut window_position = app
                            .world
                            .get_mut::<WindowPosition>(window_entity)
                            .expect("Window should have a WindowPosition component");

                        window_position.update_actual_position_from_backend(position);

                        // Event
                        let mut events = world.resource_mut::<Events<WindowMoved>>();
                        events.send(WindowMoved {
                            entity: window_entity,
                            position,
                        });
                    }
                    _ => {}
                }
            }
            event::Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                let mut mouse_motion_events = app.world.resource_mut::<Events<MouseMotion>>();
                mouse_motion_events.send(MouseMotion {
                    delta: Vec2::new(delta.0 as f32, delta.1 as f32),
                });
            }
            event::Event::Suspended => {
                winit_state.active = false;
            }
            event::Event::Resumed => {
                winit_state.active = true;
            }
            event::Event::MainEventsCleared => {
                handle_create_window_events(
                    &mut app.world,
                    event_loop,
                    &mut create_window_event_reader,
                );
                let winit_config = app.world.resource::<WinitSettings>();
                let update = if winit_state.active {
                    // True if _any_ windows are currently being focused
                    let mut windows_focused_query = app
                        .world
                        .query_filtered::<Entity, (With<Window>, With<WindowCurrentlyFocused>)>();
                    let focused = windows_focused_query
                        .iter(&app.world)
                        .collect::<Vec<Entity>>()
                        .len()
                        > 0;
                    match winit_config.update_mode(focused) {
                        UpdateMode::Continuous | UpdateMode::Reactive { .. } => true,
                        UpdateMode::ReactiveLowPower { .. } => {
                            winit_state.low_power_event
                                || winit_state.redraw_request_sent
                                || winit_state.timeout_reached
                        }
                    }
                } else {
                    false
                };
                if update {
                    winit_state.last_update = Instant::now();
                    app.update();
                }
            }
            Event::RedrawEventsCleared => {
                {
                    let winit_config = app.world.resource::<WinitSettings>();

                    // True if _any_ windows are currently being focused
                    let mut windows_focused_query = app
                        .world
                        .query_filtered::<Entity, (With<Window>, With<WindowCurrentlyFocused>)>();
                    let focused = windows_focused_query
                        .iter(&app.world)
                        .collect::<Vec<Entity>>()
                        .len()
                        > 0;

                    let now = Instant::now();
                    use UpdateMode::*;
                    *control_flow = match winit_config.update_mode(focused) {
                        Continuous => ControlFlow::Poll,
                        Reactive { max_wait } | ReactiveLowPower { max_wait } => {
                            if let Some(instant) = now.checked_add(*max_wait) {
                                ControlFlow::WaitUntil(instant)
                            } else {
                                ControlFlow::Wait
                            }
                        }
                    };
                }

                // This block needs to run after `app.update()` in `MainEventsCleared`. Otherwise,
                // we won't be able to see redraw requests until the next event, defeating the
                // purpose of a redraw request!
                let mut redraw = false;
                if let Some(app_redraw_events) = app.world.get_resource::<Events<RequestRedraw>>() {
                    if redraw_event_reader.iter(app_redraw_events).last().is_some() {
                        *control_flow = ControlFlow::Poll;
                        redraw = true;
                    }
                }
                if let Some(app_exit_events) = app.world.get_resource::<Events<AppExit>>() {
                    if app_exit_event_reader.iter(app_exit_events).last().is_some() {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                winit_state.redraw_request_sent = redraw;
            }
            _ => (),
        }
    };

    // If true, returns control from Winit back to the main Bevy loop
    if return_from_run {
        run_return(&mut event_loop, event_handler);
    } else {
        run(event_loop, event_handler);
    }
}

// TODO: Remove this is favour of the create_window system, if possible
fn handle_create_window_events(
    world: &mut World,
    event_loop: &EventLoopWindowTarget<()>,
    create_window_event_reader: &mut ManualEventReader<CreateWindow>,
) {
    // TODO: It's probably worng to be using the world directly here instead of the world-cell
    // So figure out what should be the correct approach
    let world_cell = world.cell();
    let mut winit_windows = world_cell.non_send_resource_mut::<WinitWindows>();

    // Query windows from world
    // let mut windows_query = world.query_filtered::<Entity, With<Window>>();
    // let mut windows: Vec<Entity> = windows_query.iter(world).collect();

    let create_window_events = world_cell.resource::<Events<CreateWindow>>();
    let mut window_created_events = world_cell.resource_mut::<Events<WindowCreated>>();

    for create_window_event in create_window_event_reader.iter(&create_window_events) {
        let winit_windows = winit_windows.create_window(
            event_loop,
            create_window_event.entity,
            &create_window_event.descriptor,
        );

        // TODO: Spawn all components required on the window-entity

        window_created_events.send(WindowCreated {
            entity: create_window_event.entity,
        });

        #[cfg(target_arch = "wasm32")]
        {
            let channel = world_cell.resource_mut::<web_resize::CanvasParentResizeEventChannel>();
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
