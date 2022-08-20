use bevy_ecs::entity::Entity;
use bevy_utils::{tracing::warn, HashMap};
use bevy_window::{WindowComponentsItem, WindowMode, WindowPosition, WindowResolution};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition},
    monitor::MonitorHandle,
};

#[derive(Debug, Default)]
pub struct WinitWindows {
    pub windows: HashMap<winit::window::WindowId, winit::window::Window>,
    pub window_id_to_winit: HashMap<Entity, winit::window::WindowId>,
    pub winit_to_window_id: HashMap<winit::window::WindowId, Entity>,
    // Some winit functions, such as `set_window_icon` can only be used from the main thread. If
    // they are used in another thread, the app will hang. This marker ensures `WinitWindows` is
    // only ever accessed with bevy's non-send functions and in NonSend systems.
    _not_send_sync: core::marker::PhantomData<*const ()>,
}

impl WinitWindows {
    pub fn create_window(
        &mut self,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        entity: Entity,
        components: &WindowComponentsItem,
    ) -> &winit::window::Window {
        let mut winit_window_builder = winit::window::WindowBuilder::new();

        winit_window_builder = match components.window_mode {
            WindowMode::BorderlessFullscreen => winit_window_builder.with_fullscreen(Some(
                winit::window::Fullscreen::Borderless(event_loop.primary_monitor()),
            )),
            WindowMode::Fullscreen => {
                winit_window_builder.with_fullscreen(Some(winit::window::Fullscreen::Exclusive(
                    get_best_videomode(&event_loop.primary_monitor().unwrap()),
                )))
            }
            WindowMode::SizedFullscreen => winit_window_builder.with_fullscreen(Some(
                winit::window::Fullscreen::Exclusive(get_fitting_videomode(
                    &event_loop.primary_monitor().unwrap(),
                    components.resolution.width() as u32,
                    components.resolution.height() as u32,
                )),
            )),
            _ => {
                let resolution = &components.resolution;

                if let Some(position) = winit_window_position(
                    components.position,
                    &components.resolution,
                    event_loop.available_monitors(),
                    event_loop.primary_monitor(),
                    None,
                ) {
                    winit_window_builder = winit_window_builder.with_position(position);
                }

                winit_window_builder.with_inner_size(
                    LogicalSize::new(resolution.width(), resolution.height())
                        .to_physical::<f64>(resolution.scale_factor()),
                )
            }
            .with_maximized(components.state.maximized())
            .with_resizable(components.resizable.resizable())
            .with_decorations(components.decorations.decorated())
            .with_transparent(components.transparency.transparent()),
        };

        let constraints = components.resize_constraints.check_constraints();
        let min_inner_size = LogicalSize {
            width: constraints.min_width,
            height: constraints.min_height,
        };
        let max_inner_size = LogicalSize {
            width: constraints.max_width,
            height: constraints.max_height,
        };

        let winit_window_builder =
            if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
                winit_window_builder
                    .with_min_inner_size(min_inner_size)
                    .with_max_inner_size(max_inner_size)
            } else {
                winit_window_builder.with_min_inner_size(min_inner_size)
            };

        #[allow(unused_mut)]
        let mut winit_window_builder = winit_window_builder.with_title(components.title.as_str());

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowBuilderExtWebSys;

            if let Some(selector) = &components.canvas.canvas() {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let canvas = document
                    .query_selector(&selector)
                    .expect("Cannot query for canvas element.");
                if let Some(canvas) = canvas {
                    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok();
                    winit_window_builder = winit_window_builder.with_canvas(canvas);
                } else {
                    panic!("Cannot find element: {}.", selector);
                }
            }
        }

        let winit_window = winit_window_builder.build(event_loop).unwrap();

        if components.cursor.locked() {
            match winit_window.set_cursor_grab(true) {
                Ok(_) | Err(winit::error::ExternalError::NotSupported(_)) => {}
                Err(err) => Err(err).unwrap(),
            }
        }

        winit_window.set_cursor_visible(components.cursor.visible());

        // I don't think we can do this immediately with some platforms.
        if components.state.minimized() {
            winit_window.set_minimized(true);
        }

        self.window_id_to_winit.insert(entity, winit_window.id());
        self.winit_to_window_id.insert(winit_window.id(), entity);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            if components.canvas.canvas().is_none() {
                let canvas = winit_window.canvas();

                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let body = document.body().unwrap();

                body.append_child(&canvas)
                    .expect("Append canvas to HTML body.");
            }
        }

        self.windows
            .entry(winit_window.id())
            .insert(winit_window)
            .into_mut()
    }

    /// Get the winit window that is associated with our entity.
    pub fn get_window(&self, entity: Entity) -> Option<&winit::window::Window> {
        self.window_id_to_winit
            .get(&entity)
            .and_then(|winit_id| self.windows.get(winit_id))
    }

    /// Get the entity associated with the winit window id.
    ///
    /// This is mostly just an intermediary step between us and winit.
    pub fn get_window_entity(&self, winit_id: winit::window::WindowId) -> Option<Entity> {
        self.winit_to_window_id.get(&winit_id).cloned()
    }

    /// Remove a window from winit.
    ///
    /// This should mostly just be called when the window is closing.
    pub fn remove_window(&mut self, entity: Entity) -> Option<winit::window::Window> {
        let winit_id = self.window_id_to_winit.remove(&entity)?;
        // Don't remove from winit_to_window_id, to track that we used to know about this winit window
        self.windows.remove(&winit_id)
    }
}

pub fn get_fitting_videomode(
    monitor: &winit::monitor::MonitorHandle,
    width: u32,
    height: u32,
) -> winit::monitor::VideoMode {
    let mut modes = monitor.video_modes().collect::<Vec<_>>();

    fn abs_diff(a: u32, b: u32) -> u32 {
        if a > b {
            return a - b;
        }
        b - a
    }

    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match abs_diff(a.size().width, width).cmp(&abs_diff(b.size().width, width)) {
            Equal => {
                match abs_diff(a.size().height, height).cmp(&abs_diff(b.size().height, height)) {
                    Equal => b.refresh_rate().cmp(&a.refresh_rate()),
                    default => default,
                }
            }
            default => default,
        }
    });

    modes.first().unwrap().clone()
}

pub fn get_best_videomode(monitor: &winit::monitor::MonitorHandle) -> winit::monitor::VideoMode {
    let mut modes = monitor.video_modes().collect::<Vec<_>>();
    modes.sort_by(|a, b| {
        use std::cmp::Ordering::*;
        match b.size().width.cmp(&a.size().width) {
            Equal => match b.size().height.cmp(&a.size().height) {
                Equal => b.refresh_rate().cmp(&a.refresh_rate()),
                default => default,
            },
            default => default,
        }
    });

    modes.first().unwrap().clone()
}

// Ideally we could generify this across window backends, but we only really have winit atm
// so whatever.
//
// Also ends up bringing to question the standardization of physical/logical position/sizes
// into specific types so we don't run into weird cross bugs where we use a logical position
// in a physical context.
pub fn winit_window_position(
    position: &WindowPosition,
    resolution: &WindowResolution,
    mut available_monitors: impl Iterator<Item = MonitorHandle>,
    primary_monitor: Option<MonitorHandle>,
    current_monitor: Option<MonitorHandle>,
) -> Option<PhysicalPosition<f64>> {
    match position {
        WindowPosition::Automatic => {
            /* Window manager will handle position */
            None
        }
        WindowPosition::Centered(monitor_selection) => {
            use bevy_window::MonitorSelection::*;
            let maybe_monitor = match monitor_selection {
                Current => {
                    if let None = current_monitor {
                        warn!("Can't select current monitor on window creation or cannot find current monitor!");
                    }
                    current_monitor
                }
                Primary => primary_monitor,
                Number(n) => available_monitors.nth(*n),
            };

            if let Some(monitor) = maybe_monitor {
                let screen_size = monitor.size();

                let scale_factor = resolution.scale_factor();

                // Logical to physical window size
                let (width, height): (u32, u32) =
                    LogicalSize::new(resolution.width(), resolution.height())
                        .to_physical::<u32>(scale_factor)
                        .into();

                let position = PhysicalPosition {
                    x: screen_size.width.saturating_sub(width) as f64 / 2.
                        + monitor.position().x as f64,
                    y: screen_size.height.saturating_sub(height) as f64 / 2.
                        + monitor.position().y as f64,
                };

                Some(position)
            } else {
                warn!("Couldn't get monitor selected with: {monitor_selection:?}");
                None
            }
        }
        WindowPosition::At(position) => Some(
            LogicalPosition::new(position[0] as f64, position[1] as f64)
                .to_physical::<f64>(resolution.scale_factor()),
        ),
    }
}

// WARNING: this only works under the assumption that wasm runtime is single threaded
#[cfg(target_arch = "wasm32")]
unsafe impl Send for WinitWindows {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for WinitWindows {}
