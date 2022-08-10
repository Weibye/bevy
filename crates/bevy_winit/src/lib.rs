mod converters;
mod system;
#[cfg(target_arch = "wasm32")]
mod web_resize;
mod winit_config;
mod winit_windows;

use core::panic;

use bevy_ecs::system::{SystemParam, SystemState};
use bevy_utils::tracing::info;
use system::{
    create_window_system, update_cursor, update_cursor_position, update_resize_constraints,
    update_resolution, update_title, update_window_mode, update_window_position,
    update_window_state, window_destroyed,
};

pub use winit_config::*;
pub use winit_windows::*;

use bevy_app::{App, AppExit, CoreStage, Plugin};
use bevy_ecs::event::{Events, ManualEventReader};
use bevy_ecs::prelude::*;
use bevy_input::{
    keyboard::KeyboardInput,
    mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel},
    touch::TouchInput,
};
use bevy_math::{ivec2, DVec2, Vec2};
use bevy_utils::{
    tracing::{trace, warn},
    Instant,
};
use bevy_window::{
    CursorEntered, CursorLeft, CursorMoved, FileDragAndDrop, ModifiesWindows, ReceivedCharacter,
    RequestRedraw, Window, WindowBackendScaleFactorChanged, WindowCloseRequested, WindowComponents,
    WindowComponentsMut, WindowComponentsMutItem, WindowFocus, WindowFocused, WindowMoved,
    WindowResized, WindowScaleFactorChanged, WindowState,
};

use winit::{
    event::{self, DeviceEvent, Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
};

#[derive(Default)]
pub struct WinitPlugin;

impl Plugin for WinitPlugin {
    fn build(&self, app: &mut App) {
        let event_loop = EventLoop::new();
        app.insert_non_send_resource(event_loop);

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
                    .with_system(update_window_state)
                    .with_system(update_window_position)
                    .with_system(update_resolution)
                    .with_system(update_cursor)
                    .with_system(update_cursor_position)
                    .with_system(update_resize_constraints),
            )
            .add_system_to_stage(CoreStage::Last, window_destroyed);

        #[cfg(target_arch = "wasm32")]
        app.add_plugin(web_resize::CanvasParentResizePlugin);

        let mut system_state: SystemState<(
            Commands,
            NonSendMut<EventLoop<()>>,
            Query<(Entity, WindowComponents), Added<Window>>,
            NonSendMut<WinitWindows>,
        )> = SystemState::from_world(&mut app.world);

        {
            let (commands, event_loop, new_windows, winit_windows) =
                system_state.get_mut(&mut app.world);

            // Here we need to create a winit-window and give it a WindowHandle which the renderer can use.
            // It needs to be spawned before the start of the startup-stage, so we cannot use a regular system.
            // Instead we need to create the window and spawn it using direct world access
            create_window_system(commands, &**event_loop, new_windows, winit_windows);
        }

        system_state.apply(&mut app.world);
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

#[derive(SystemParam)]
struct WindowEvents<'w, 's> {
    window_resized: EventWriter<'w, 's, WindowResized>,
    window_close_requested: EventWriter<'w, 's, WindowCloseRequested>,
    window_scale_factor_changed: EventWriter<'w, 's, WindowScaleFactorChanged>,
    window_backend_scale_factor_changed: EventWriter<'w, 's, WindowBackendScaleFactorChanged>,
    window_focused: EventWriter<'w, 's, WindowFocused>,
    window_moved: EventWriter<'w, 's, WindowMoved>,
}

#[derive(SystemParam)]
struct InputEvents<'w, 's> {
    keyboard_input: EventWriter<'w, 's, KeyboardInput>,
    character_input: EventWriter<'w, 's, ReceivedCharacter>,
    mouse_button_input: EventWriter<'w, 's, MouseButtonInput>,
    mouse_wheel_input: EventWriter<'w, 's, MouseWheel>,
    touch_input: EventWriter<'w, 's, TouchInput>,
    // mouse_motion: EventWriter<'w, 's, MouseMotion>,
}

#[derive(SystemParam)]
struct CursorEvents<'w, 's> {
    cursor_moved: EventWriter<'w, 's, CursorMoved>,
    cursor_entered: EventWriter<'w, 's, CursorEntered>,
    cursor_left: EventWriter<'w, 's, CursorLeft>,
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

// #[derive(Default)]
// struct WinitCreateWindowReader(ManualEventReader<CreateWindow>);

// TODO: Refactor this to work with new pattern
pub fn winit_runner(mut app: App) {
    // TODO: Understand what removing and adding this does
    let mut event_loop = app
        .world
        .remove_non_send_resource::<EventLoop<()>>()
        .unwrap();
    // let mut create_window_event_reader = app
    //     .world
    //     .remove_resource::<WinitCreateWindowReader>()
    //     .unwrap()
    //     .0;
    let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();
    let mut redraw_event_reader = ManualEventReader::<RequestRedraw>::default();
    let mut winit_state = WinitPersistentState::default();
    app.world
        .insert_non_send_resource(event_loop.create_proxy());

    let return_from_run = app.world.resource::<WinitSettings>().return_from_run;

    trace!("Entering winit event loop");

    let mut create_window_system_state: SystemState<(
        Commands,
        Query<(Entity, WindowComponents), Added<Window>>,
        NonSendMut<WinitWindows>,
        Res<WinitSettings>,
        Query<&WindowFocus, With<Window>>,
    )> = SystemState::from_world(&mut app.world);

    let event_handler = move |event: Event<()>,
                              event_loop: &EventLoopWindowTarget<()>,
                              control_flow: &mut ControlFlow| {
        if let Some(app_exit_events) = app.world.get_resource::<Events<AppExit>>() {
            if app_exit_event_reader.iter(app_exit_events).last().is_some() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        match event {
            event::Event::NewEvents(start) => {
                // Fetch from the world
                let mut system_state: SystemState<(
                    Res<WinitSettings>,
                    Query<&WindowFocus, With<Window>>,
                )> = SystemState::new(&mut app.world);

                let (winit_config, window_focused_query) = system_state.get(&mut app.world);

                let any_window_focused =
                    window_focused_query.iter().any(|focused| focused.focused());

                // Check if either the `WaitUntil` timeout was triggered by winit, or that same
                // amount of time has elapsed since the last app update. This manual check is needed
                // because we don't know if the criteria for an app update were met until the end of
                // the frame.
                let auto_timeout_reached = matches!(start, StartCause::ResumeTimeReached { .. });
                let now = Instant::now();
                let manual_timeout_reached = match winit_config.update_mode(any_window_focused) {
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
                // Fetch and prepare details from the world
                let mut system_state: SystemState<(
                    NonSend<WinitWindows>,
                    Query<WindowComponentsMut, With<Window>>,
                    WindowEvents,
                    InputEvents,
                    CursorEvents,
                    EventWriter<FileDragAndDrop>,
                )> = SystemState::new(&mut app.world);
                let (
                    winit_windows,
                    mut window_query,
                    mut window_events,
                    mut input_events,
                    mut cursor_events,
                    mut file_drag_and_drop_events,
                ) = system_state.get_mut(&mut app.world);

                // Entity of this window
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

                // Reference to the Winit-window
                let winit_window = if let Some(window) = winit_windows.get_window(window_entity) {
                    window
                } else {
                    warn!(
                        "Skipped event for non-existent Winit Window Id {:?}",
                        winit_window_id
                    );
                    return;
                };

                if window_query.get(window_entity).is_err() {
                    warn!(
                        "Skipped event for non-existent Window Id {:?}",
                        winit_window_id
                    );
                    return;
                }

                winit_state.low_power_event = true;

                match event {
                    WindowEvent::Resized(size) => {
                        if let Ok(mut window) = window_query.get_mut(window_entity) {
                            // Update component
                            window
                                .resolution
                                .set_physical_resolution(size.width, size.height);

                            // Send event to notify change
                            window_events.window_resized.send(WindowResized {
                                entity: window_entity,
                                width: window.resolution.width(),
                                height: window.resolution.height(),
                            });
                        } else {
                            // TODO: Helpful panic comment
                            panic!("Window does not have a valid WindowResolution component");
                        }
                    }
                    WindowEvent::CloseRequested => {
                        window_events
                            .window_close_requested
                            .send(WindowCloseRequested {
                                entity: window_entity,
                            });
                    }
                    WindowEvent::KeyboardInput { ref input, .. } => {
                        input_events
                            .keyboard_input
                            .send(converters::convert_keyboard_input(input));
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let inner_size = winit_window.inner_size();

                        if let Ok(mut window) = window_query.get_mut(window_entity) {
                            // Components

                            let physical_position = DVec2::new(
                                position.x,
                                // Flip the coordinate space from winit's context to our context.
                                inner_size.height as f64 - position.y,
                            );

                            window.cursor_position.set(Some(physical_position));

                            // Event
                            cursor_events.cursor_moved.send(CursorMoved {
                                entity: window_entity,
                                position: (physical_position / window.resolution.scale_factor())
                                    .as_vec2(),
                            });
                        } else {
                            warn!(
                                "could not set cursor position of window: {:?}",
                                window_entity
                            );
                        }
                    }
                    WindowEvent::CursorEntered { .. } => {
                        cursor_events.cursor_entered.send(CursorEntered {
                            entity: window_entity,
                        });
                    }
                    WindowEvent::CursorLeft { .. } => {
                        // Component
                        if let Ok(mut window) = window_query.get_mut(window_entity) {
                            window.cursor_position.set(None);

                            // Event
                            cursor_events.cursor_left.send(CursorLeft {
                                entity: window_entity,
                            });
                        } else {
                            warn!(
                                "could not set cursor position of window: {:?}",
                                window_entity
                            );
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        input_events.mouse_button_input.send(MouseButtonInput {
                            button: converters::convert_mouse_button(button),
                            state: converters::convert_element_state(state),
                        });
                    }
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        event::MouseScrollDelta::LineDelta(x, y) => {
                            input_events.mouse_wheel_input.send(MouseWheel {
                                unit: MouseScrollUnit::Line,
                                x,
                                y,
                            });
                        }
                        event::MouseScrollDelta::PixelDelta(p) => {
                            input_events.mouse_wheel_input.send(MouseWheel {
                                unit: MouseScrollUnit::Pixel,
                                x: p.x as f32,
                                y: p.y as f32,
                            });
                        }
                    },
                    WindowEvent::Touch(touch) => {
                        if let Ok(window) = window_query.get(window_entity) {
                            let mut location =
                                touch.location.to_logical(window.resolution.scale_factor());

                            // On a mobile window, the start is from the top while on PC/Linux/OSX from
                            // bottom
                            if cfg!(target_os = "android") || cfg!(target_os = "ios") {
                                location.y = window.resolution.height() - location.y;
                            }

                            // Event
                            input_events
                                .touch_input
                                .send(converters::convert_touch_input(touch, location));
                        } else {
                            warn!(
                                "could not get resolution for touch event on window: {:?}",
                                window_entity
                            );
                        }
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        input_events.character_input.send(ReceivedCharacter {
                            entity: window_entity,
                            char: c,
                        });
                    }
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size,
                    } => {
                        window_events.window_backend_scale_factor_changed.send(
                            WindowBackendScaleFactorChanged {
                                entity: window_entity,
                                scale_factor,
                            },
                        );

                        // Components
                        let mut window = window_query
                            .get_mut(window_entity)
                            .expect("expected window components");

                        let prior_factor = window.resolution.scale_factor();
                        window.resolution.set_scale_factor(scale_factor);
                        let new_factor = window.resolution.scale_factor();

                        if let Some(forced_factor) = window.resolution.scale_factor_override() {
                            // If there is a scale factor override, then force that to be used
                            // Otherwise, use the OS suggested size
                            // We have already told the OS about our resize constraints, so
                            // the new_inner_size should take those into account
                            *new_inner_size = winit::dpi::LogicalSize::new(
                                window.resolution.requested_width(),
                                window.resolution.requested_height(),
                            )
                            .to_physical::<u32>(forced_factor);
                            // TODO: Should this not trigger a WindowsScaleFactorChanged?
                        } else if approx::relative_ne!(new_factor, prior_factor) {
                            // Trigger a change event if they are approximately different
                            window_events.window_scale_factor_changed.send(
                                WindowScaleFactorChanged {
                                    entity: window_entity,
                                    scale_factor,
                                },
                            );
                        }

                        let new_logical_width = new_inner_size.width as f64 / new_factor;
                        let new_logical_height = new_inner_size.height as f64 / new_factor;
                        if approx::relative_ne!(window.resolution.width(), new_logical_width)
                            || approx::relative_ne!(window.resolution.height(), new_logical_height)
                        {
                            window_events.window_resized.send(WindowResized {
                                entity: window_entity,
                                width: new_logical_width,
                                height: new_logical_height,
                            });
                        }
                        window
                            .resolution
                            .set_physical_resolution(new_inner_size.width, new_inner_size.height);
                    }
                    WindowEvent::Focused(focused) => {
                        let mut window = window_query
                            .get_mut(window_entity)
                            .expect("expected window components");

                        // Component
                        window.focus.set(focused);

                        // Event
                        window_events.window_focused.send(WindowFocused {
                            entity: window_entity,
                            focused,
                        });
                    }
                    WindowEvent::DroppedFile(path_buf) => {
                        file_drag_and_drop_events.send(FileDragAndDrop::DroppedFile {
                            entity: window_entity,
                            path_buf,
                        });
                    }
                    WindowEvent::HoveredFile(path_buf) => {
                        file_drag_and_drop_events.send(FileDragAndDrop::HoveredFile {
                            entity: window_entity,
                            path_buf,
                        });
                    }
                    WindowEvent::HoveredFileCancelled => {
                        file_drag_and_drop_events.send(FileDragAndDrop::HoveredFileCancelled {
                            entity: window_entity,
                        });
                    }
                    WindowEvent::Moved(position) => {
                        let position = ivec2(position.x, position.y);

                        // Component
                        let mut window = window_query
                            .get_mut(window_entity)
                            .expect("Window should have a WindowPosition component");
                        window.position.set(position);

                        // Event
                        window_events.window_moved.send(WindowMoved {
                            entity: window_entity,
                            position,
                        });
                    }
                    _ => {}
                }

                // We probably don't need to do this on every window event, but they are uncommon enough
                // that I think it is fine and would reduce maintenance burden here.
                if let Ok(mut window) = window_query.get_mut(window_entity) {
                    let should_be = if winit_window.is_maximized() {
                        WindowState::Maximized
                    } else {
                        if window.resolution.zero() {
                            WindowState::Minimized
                        } else {
                            WindowState::Normal
                        }
                    };

                    if *window.state != should_be {
                        *window.state = should_be;
                    }
                }
            }
            event::Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                let mut system_state: SystemState<EventWriter<MouseMotion>> =
                    SystemState::new(&mut app.world);
                let mut mouse_motion = system_state.get_mut(&mut app.world);

                mouse_motion.send(MouseMotion {
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
                let (commands, new_windows, winit_windows, winit_config, window_focused_query) =
                    create_window_system_state.get_mut(&mut app.world);

                // Responsible for creating new windows
                create_window_system(commands, event_loop, new_windows, winit_windows);

                let update = if winit_state.active {
                    // True if _any_ windows are currently being focused
                    // TODO: Do we need to fetch windows again since new ones might have been created and they might be focused?
                    let focused = window_focused_query.iter().any(|focused| focused.focused());
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
                    // Fetch from world
                    let mut system_state: SystemState<(
                        Res<WinitSettings>,
                        Query<&WindowFocus, With<Window>>,
                    )> = SystemState::new(&mut app.world);

                    let (winit_config, window_focused_query) = system_state.get(&mut app.world);

                    // True if _any_ windows are currently being focused
                    let focused = window_focused_query.iter().any(|focused| focused.focused());

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

                winit_state.redraw_request_sent = redraw;
            }

            _ => (),
        }

        create_window_system_state.apply(&mut app.world);
    };

    // If true, returns control from Winit back to the main Bevy loop
    if return_from_run {
        run_return(&mut event_loop, event_handler);
    } else {
        run(event_loop, event_handler);
    }
}
