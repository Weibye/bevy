use bevy_ecs::system::Resource;
use std::borrow::Cow;

use bevy_ecs::{
    entity::Entity,
    prelude::{Bundle, Component},
    query::WorldQuery,
};
use bevy_math::{DVec2, IVec2, UVec2, Vec2};
use bevy_reflect::{FromReflect, Reflect};
use bevy_utils::{tracing::warn, Uuid};
use raw_window_handle::RawWindowHandle;

use crate::CursorIcon;
use crate::{raw_window_handle::RawWindowHandleWrapper, WindowFocused};

/// Presentation mode for a window.
///
/// The presentation mode specifies when a frame is presented to the window. The `Fifo`
/// option corresponds to a traditional `VSync`, where the framerate is capped by the
/// display refresh rate. Both `Immediate` and `Mailbox` are low-latency and are not
/// capped by the refresh rate, but may not be available on all platforms. Tearing
/// may be observed with `Immediate` mode, but will not be observed with `Mailbox` or
/// `Fifo`.
///
/// `Immediate` or `Mailbox` will gracefully fallback to `Fifo` when unavailable.
///
/// The presentation mode may be declared in the [`WindowDescriptor`](WindowDescriptor::present_mode)
/// or updated on a [`Window`](Window::set_present_mode).
#[repr(C)]
#[derive(Copy, Clone, Component, Debug, PartialEq, Eq, Hash)]
#[doc(alias = "vsync")]
pub enum PresentMode {
    /// Chooses FifoRelaxed -> Fifo based on availability.
    ///
    /// Because of the fallback behavior, it is supported everywhere.
    AutoVsync = 0,
    /// Chooses Immediate -> Mailbox -> Fifo (on web) based on availability.
    ///
    /// Because of the fallback behavior, it is supported everywhere.
    AutoNoVsync = 1,
    /// The presentation engine does **not** wait for a vertical blanking period and
    /// the request is presented immediately. This is a low-latency presentation mode,
    /// but visible tearing may be observed. Will fallback to `Fifo` if unavailable on the
    /// selected platform and backend. Not optimal for mobile.
    ///
    /// Selecting this variant will panic if not supported, it is preferred to use
    /// [`PresentMode::AutoNoVsync`].
    Immediate = 2,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image, but frames may be submitted without delay. This is a low-latency
    /// presentation mode and visible tearing will **not** be observed. Will fallback to `Fifo`
    /// if unavailable on the selected platform and backend. Not optimal for mobile.
    ///
    /// Selecting this variant will panic if not supported, it is preferred to use
    /// [`PresentMode::AutoNoVsync`].
    Mailbox = 3,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image. The framerate will be capped at the display refresh rate,
    /// corresponding to the `VSync`. Tearing cannot be observed. Optimal for mobile.
    Fifo = 4, // NOTE: The explicit ordinal values mirror wgpu.
}

impl Default for PresentMode {
    fn default() -> Self {
        PresentMode::Fifo
    }
}

/// Defines the way a window is displayed
#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub enum WindowMode {
    /// Creates a window that uses the given size
    Windowed,
    /// Creates a borderless window that uses the full size of the screen
    BorderlessFullscreen,
    /// Creates a fullscreen window that will render at desktop resolution. The app will use the closest supported size
    /// from the given size and scale it to fit the screen.
    SizedFullscreen,
    /// Creates a fullscreen window that uses the maximum supported size
    Fullscreen,
}

impl Default for WindowMode {
    fn default() -> Self {
        WindowMode::Windowed
    }
}

/// Define how a window will be created and how it will behave.
#[derive(Default, Bundle, Debug, Clone)]
pub struct WindowBundle {
    pub window: Window,
    pub cursor: Cursor,
    pub cursor_position: CursorPosition,
    pub present_mode: PresentMode,
    pub mode: WindowMode,
    pub position: WindowPosition,
    pub resolution: WindowResolution,
    pub title: WindowTitle,
    // Maybe default this when using wasm?
    //pub canvas: WindowCanvas,
    pub resize_constraints: WindowResizeConstraints,
    pub resizable: WindowResizable,
    pub decorated: WindowDecorated,
}

#[derive(WorldQuery)]
pub struct WindowComponents<'a> {
    pub entity: Entity,
    pub window: &'a Window,
    pub cursor: &'a Cursor,
    pub cursor_position: &'a CursorPosition,
    pub present_mode: &'a PresentMode,
    pub window_mode: &'a WindowMode,
    pub position: &'a WindowPosition,
    pub resolution: &'a WindowResolution,
    pub title: &'a WindowTitle,
    pub resize_constraints: &'a WindowResizeConstraints,

    pub resizable: Option<&'a WindowResizable>,
    pub decorated: Option<&'a WindowDecorated>,
    pub transparent: Option<&'a WindowTransparent>,
}

/// The size limits on a window.
///
/// These values are measured in logical pixels, so the user's
/// scale factor does affect the size limits on the window.
/// Please note that if the window is resizable, then when the window is
/// maximized it may have a size outside of these limits. The functionality
/// required to disable maximizing is not yet exposed by winit.
#[derive(Debug, Clone, Copy, Component)]
pub struct WindowResizeConstraints {
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

impl Default for WindowResizeConstraints {
    fn default() -> Self {
        Self {
            min_width: 180.,
            min_height: 120.,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        }
    }
}

impl WindowResizeConstraints {
    #[must_use]
    pub fn check_constraints(&self) -> Self {
        let WindowResizeConstraints {
            mut min_width,
            mut min_height,
            mut max_width,
            mut max_height,
        } = self;
        min_width = min_width.max(1.);
        min_height = min_height.max(1.);
        if max_width < min_width {
            warn!(
                "The given maximum width {} is smaller than the minimum width {}",
                max_width, min_width
            );
            max_width = min_width;
        }
        if max_height < min_height {
            warn!(
                "The given maximum height {} is smaller than the minimum height {}",
                max_height, min_height
            );
            max_height = min_height;
        }
        WindowResizeConstraints {
            min_width,
            min_height,
            max_width,
            max_height,
        }
    }
}

/// A marker component on an entity that is a window
#[derive(Default, Debug, Component, Copy, Clone)]
pub struct Window;

#[derive(Debug, Component, Copy, Clone)]
pub struct Cursor {
    icon: CursorIcon,
    visible: bool,
    locked: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            icon: CursorIcon::Default,
            visible: true,
            locked: false,
        }
    }
}

impl Cursor {
    pub fn new(icon: CursorIcon, visible: bool, locked: bool) -> Self {
        Self {
            icon,
            visible,
            locked,
        }
    }

    #[inline]
    pub fn icon(&self) -> CursorIcon {
        self.icon
    }

    #[inline]
    pub fn visible(&self) -> bool {
        self.visible
    }

    #[inline]
    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn set_icon(&mut self, icon: CursorIcon) {
        self.icon = icon;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }
}

#[derive(Default, Debug, Component, Clone)]
pub struct CursorPosition {
    /// Cursor position if it is inside of the window.
    physical_cursor_position: Option<DVec2>,
}

impl CursorPosition {
    pub fn new(physical_cursor_position: Option<DVec2>) -> Self {
        Self {
            physical_cursor_position,
        }
    }

    /// The current mouse position, in physical pixels.
    #[inline]
    pub fn position(&self) -> Option<DVec2> {
        self.physical_cursor_position
    }

    pub fn set(&mut self, position: Option<DVec2>) {
        self.physical_cursor_position = position;
    }
}

#[derive(Component)]
pub struct WindowHandle {
    raw_window_handle: RawWindowHandleWrapper,
}

impl WindowHandle {
    pub fn new(raw_window_handle: RawWindowHandle) -> Self {
        Self {
            raw_window_handle: RawWindowHandleWrapper::new(raw_window_handle),
        }
    }

    pub fn raw_window_handle(&self) -> RawWindowHandleWrapper {
        self.raw_window_handle.clone()
    }
}

/// Defines where window should be placed at on creation.
#[derive(Debug, Clone, Copy, Component)]
pub enum WindowPosition {
    /// Position will be set by the window manager
    Automatic,
    /// Window will be centered on the selected monitor
    ///
    /// Note that this does not account for window decorations.
    Centered(MonitorSelection),
    /// The window's top-left corner will be placed at the specified position (in pixels)
    ///
    /// (0,0) represents top-left corner of screen space.
    At(IVec2),
}

impl Default for WindowPosition {
    fn default() -> Self {
        WindowPosition::Automatic
    }
}

impl WindowPosition {
    pub fn new(position: IVec2) -> Self {
        Self::At(position)
    }

    /// The window's client position in physical pixels.
    #[inline]
    pub fn position(&self) -> Option<IVec2> {
        match self {
            Self::At(position) => Some(*position),
            _ => None,
        }
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn update_actual_position_from_backend(&mut self, position: IVec2) {
        *self = WindowPosition::At(position);
    }
}

/// ## Window Sizes
///
/// There are three sizes associated with a window. The physical size which is
/// the height and width in physical pixels on the monitor. The logical size
/// which is the physical size scaled by an operating system provided factor to
/// account for monitors with differing pixel densities or user preference. And
/// the requested size, measured in logical pixels, which is the value submitted
/// to the API when creating the window, or requesting that it be resized.
///
/// The actual size, in logical pixels, of the window may not match the
/// requested size due to operating system limits on the window size, or the
/// quantization of the logical size when converting the physical size to the
/// logical size through the scaling factor.
// TODO: Make sure this is used correctly
#[derive(Component, Debug, Clone)]
pub struct WindowResolution {
    requested_width: f32,
    requested_height: f32,
    physical_width: u32,
    physical_height: u32,
    scale_factor_override: Option<f64>,
    scale_factor: f64,
}

impl Default for WindowResolution {
    fn default() -> Self {
        WindowResolution {
            requested_width: 1280.,
            requested_height: 720.,
            physical_width: 1280,
            physical_height: 720,
            scale_factor_override: None,
            scale_factor: 1.0,
        }
    }
}

impl WindowResolution {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            requested_width: width,
            requested_height: height,
            physical_width: width as u32,
            physical_height: height as u32,
            ..Default::default()
        }
    }

    /// The ratio of physical pixels to logical pixels
    ///
    /// `physical_pixels = logical_pixels * scale_factor`
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor_override
            .unwrap_or(self.base_scale_factor())
    }

    /// The window scale factor as reported by the window backend.
    ///
    /// This value is unaffected by [`scale_factor_override`](Window::scale_factor_override).
    #[inline]
    pub fn base_scale_factor(&self) -> f64 {
        self.scale_factor
    }
    /// The scale factor set with [`set_scale_factor_override`](Window::set_scale_factor_override).
    ///
    /// This value may be different from the scale factor reported by the window backend.
    #[inline]
    pub fn scale_factor_override(&self) -> Option<f64> {
        self.scale_factor_override
    }

    /// The current logical width of the window's client area.
    #[inline]
    pub fn width(&self) -> f32 {
        (self.physical_width as f64 / self.scale_factor()) as f32
    }

    /// The current logical height of the window's client area.
    #[inline]
    pub fn height(&self) -> f32 {
        (self.physical_height as f64 / self.scale_factor()) as f32
    }

    /// The requested window client area width in logical pixels from window
    /// creation or the last call to [`set_resolution`](Window::set_resolution).
    ///
    /// This may differ from the actual width depending on OS size limits and
    /// the scaling factor for high DPI monitors.
    #[inline]
    pub fn requested_width(&self) -> f32 {
        self.requested_width
    }

    /// The requested window client area height in logical pixels from window
    /// creation or the last call to [`set_resolution`](Window::set_resolution).
    ///
    /// This may differ from the actual width depending on OS size limits and
    /// the scaling factor for high DPI monitors.
    #[inline]
    pub fn requested_height(&self) -> f32 {
        self.requested_height
    }

    /// The window's client area width in physical pixels.
    #[inline]
    pub fn physical_width(&self) -> u32 {
        self.physical_width
    }

    /// The window's client area height in physical pixels.
    #[inline]
    pub fn physical_height(&self) -> u32 {
        self.physical_height
    }

    /// Set the window's scale factor, this may get overriden by the backend.
    #[inline]
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    /// Set the window's scale factor, this will be used over what the backend decides.
    #[inline]
    pub fn set_scale_factor_override(&mut self, scale_factor_override: Option<f64>) {
        self.scale_factor_override = scale_factor_override;
    }

    /// Set the window's logical resolution.
    #[inline]
    pub fn set_resolution(&mut self, width: f32, height: f32) {
        self.requested_width = width;
        self.requested_height = height;
    }

    /// Set the window's physical resolution in pixels.
    #[inline]
    pub fn set_physical_resolution(&mut self, physical_width: u32, physical_height: u32) {
        self.physical_width = physical_width;
        self.physical_height = physical_height;
    }
}

#[derive(Component, Debug, Clone)]
pub struct WindowTitle {
    title: Cow<'static, str>,
}

impl Default for WindowTitle {
    fn default() -> Self {
        WindowTitle::new("Bevy App")
    }
}

impl WindowTitle {
    /// Creates a new [`WindowTitle`] from any string-like type.
    pub fn new(title: impl Into<Cow<'static, str>>) -> Self {
        WindowTitle {
            title: title.into(),
        }
    }

    /// Sets the window's title.
    #[inline(always)]
    pub fn set(&mut self, title: impl Into<Cow<'static, str>>) {
        *self = WindowTitle::new(title.into());
    }

    /// Gets the title of the window as a `&str`.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.title
    }
}

#[derive(Default, Component, Debug, Clone)]
pub struct WindowDecorated;

#[derive(Default, Component, Debug, Clone)]
pub struct WindowCurrentlyFocused;

#[derive(Default, Component, Debug, Clone)]
pub struct WindowResizable;

#[derive(Default, Component, Debug, Clone)]
pub struct WindowTransparent;

#[derive(Default, Component, Debug, Clone)]
pub struct WindowMinimized;

#[derive(Default, Component, Debug, Clone)]
pub struct WindowMaximized;

#[derive(Component, Debug, Clone)]
pub struct WindowCanvas {
    canvas: Option<String>,
    fit_canvas_to_parent: bool,
}

impl WindowCanvas {
    pub fn new(canvas: Option<String>, fit_canvas_to_parent: bool) -> Self {
        Self {
            canvas,
            fit_canvas_to_parent,
        }
    }

    /// The "html canvas" element selector. If set, this selector will be used to find a matching html canvas element,
    /// rather than creating a new one.   
    /// Uses the [CSS selector format](https://developer.mozilla.org/en-US/docs/Web/API/Document/querySelector).
    ///
    /// This value has no effect on non-web platforms.
    #[inline]
    pub fn canvas(&self) -> Option<&str> {
        self.canvas.as_deref()
    }

    /// Whether or not to fit the canvas element's size to its parent element's size.
    ///
    /// **Warning**: this will not behave as expected for parents that set their size according to the size of their
    /// children. This creates a "feedback loop" that will result in the canvas growing on each resize. When using this
    /// feature, ensure the parent's size is not affected by its children.
    ///
    /// This value has no effect on non-web platforms.
    #[inline]
    pub fn fit_canvas_to_parent(&self) -> bool {
        self.fit_canvas_to_parent
    }
}

/// Defines which monitor to use.
#[derive(Debug, Clone, Copy)]
pub enum MonitorSelection {
    /// Uses current monitor of the window.
    Current,
    /// Uses primary monitor of the system.
    Primary,
    /// Uses monitor with the specified index.
    Number(usize),
}
<<<<<<< HEAD

<<<<<<< HEAD
/// Describes the information needed for creating a window.
///
/// This should be set up before adding the [`WindowPlugin`](crate::WindowPlugin).
/// Most of these settings can also later be configured through the [`Window`](crate::Window) resource.
///
/// See [`examples/window/window_settings.rs`] for usage.
///
/// [`examples/window/window_settings.rs`]: https://github.com/bevyengine/bevy/blob/latest/examples/window/window_settings.rs
#[derive(Resource, Debug, Clone)]
pub struct WindowDescriptor {
    /// The requested logical width of the window's client area.
    ///
    /// May vary from the physical width due to different pixel density on different monitors.
    pub width: f32,
    /// The requested logical height of the window's client area.
    ///
    /// May vary from the physical height due to different pixel density on different monitors.
    pub height: f32,
    /// The position on the screen that the window will be placed at.
    pub position: WindowPosition,
    /// Sets minimum and maximum resize limits.
    pub resize_constraints: WindowResizeConstraints,
    /// Overrides the window's ratio of physical pixels to logical pixels.
    ///
    /// If there are some scaling problems on X11 try to set this option to `Some(1.0)`.
    pub scale_factor_override: Option<f64>,
    /// Sets the title that displays on the window top bar, on the system task bar and other OS specific places.
    ///
    /// ## Platform-specific
    /// - Web: Unsupported.
    pub title: String,
    /// Controls when a frame is presented to the screen.
    #[doc(alias = "vsync")]
    /// The window's [`PresentMode`].
    ///
    /// Used to select whether or not VSync is used
    pub present_mode: PresentMode,
    /// Sets whether the window is resizable.
    ///
    /// ## Platform-specific
    /// - iOS / Android / Web: Unsupported.
    pub resizable: bool,
    /// Sets whether the window should have borders and bars.
    pub decorations: bool,
    /// Sets whether the cursor is visible when the window has focus.
    pub cursor_visible: bool,
    /// Sets whether the window locks the cursor inside its borders when the window has focus.
    pub cursor_locked: bool,
    /// Sets the [`WindowMode`](crate::WindowMode).
    pub mode: WindowMode,
    /// Sets whether the background of the window should be transparent.
    ///
    /// ## Platform-specific
    /// - iOS / Android / Web: Unsupported.
    /// - macOS X: Not working as expected.
    /// - Windows 11: Not working as expected
    /// macOS X transparent works with winit out of the box, so this issue might be related to: <https://github.com/gfx-rs/wgpu/issues/687>
    /// Windows 11 is related to <https://github.com/rust-windowing/winit/issues/2082>
    pub transparent: bool,
    /// The "html canvas" element selector.
    ///
    /// If set, this selector will be used to find a matching html canvas element,
    /// rather than creating a new one.   
    /// Uses the [CSS selector format](https://developer.mozilla.org/en-US/docs/Web/API/Document/querySelector).
    ///
    /// This value has no effect on non-web platforms.
    pub canvas: Option<String>,
    /// Whether or not to fit the canvas element's size to its parent element's size.
    ///
    /// **Warning**: this will not behave as expected for parents that set their size according to the size of their
    /// children. This creates a "feedback loop" that will result in the canvas growing on each resize. When using this
    /// feature, ensure the parent's size is not affected by its children.
    ///
    /// This value has no effect on non-web platforms.
    pub fit_canvas_to_parent: bool,
}
