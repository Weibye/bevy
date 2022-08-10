use std::path::PathBuf;

use bevy_ecs::entity::Entity;
use bevy_math::{IVec2, Vec2};

/// A window event that is sent whenever a window's logical size has changed.
#[derive(Debug, Clone)]
pub struct WindowResized {
    pub entity: Entity,
    /// The new logical width of the window
    pub width: f64,
    /// The new logical height of the window.
    pub height: f64,
}

// TODO: This would redraw all windows ? If yes, update docs to reflect this
/// An event that indicates the window should redraw, even if its control flow is set to `Wait` and
/// there have been no window events.
#[derive(Debug, Clone)]
pub struct RequestRedraw;

/// An event that is sent whenever a new window is created.
///
/// To create a new window, send a [`CreateWindow`] event - this
/// event will be sent in the handler for that event.
#[derive(Debug, Clone)]
pub struct WindowCreated {
    pub entity: Entity,
}

/// An event that is sent whenever the operating systems requests that a window
/// be closed. This will be sent when the close button of the window is pressed.
///
/// If the default [`WindowPlugin`] is used, these events are handled
/// by [closing] the corresponding [`Window`].  
/// To disable this behaviour, set `close_when_requested` on the [`WindowPlugin`]
/// to `false`.
///
/// [`WindowPlugin`]: crate::WindowPlugin
/// [`Window`]: crate::Window
#[derive(Debug, Clone)]
pub struct WindowCloseRequested {
    pub entity: Entity,
}

/// An event that is sent whenever a window is closed. This will be sent by the
/// handler for [`WindowCloseRequested`] or similar.
#[derive(Debug, Clone)]
pub struct WindowClosed {
    pub entity: Entity,
}
/// An event reporting that the mouse cursor has moved on a window.
///
/// The event is sent only if the cursor is over one of the application's windows.
/// It is the translated version of [`WindowEvent::CursorMoved`] from the `winit` crate.
///
/// Not to be confused with the [`MouseMotion`] event from `bevy_input`.
///
/// [`WindowEvent::CursorMoved`]: https://docs.rs/winit/latest/winit/event/enum.WindowEvent.html#variant.CursorMoved
/// [`MouseMotion`]: bevy_input::mouse::MouseMotion
#[derive(Debug, Clone)]
pub struct CursorMoved {
    /// Window that the cursor moved inside.
    pub entity: Entity,
    /// The cursor position in logical pixels.
    pub position: Vec2,
}

/// An event that is sent whenever the user's cursor enters a window.
#[derive(Debug, Clone)]
pub struct CursorEntered {
    /// Window that the cursor entered.
    pub entity: Entity,
}

/// An event that is sent whenever the user's cursor leaves a window.
#[derive(Debug, Clone)]
pub struct CursorLeft {
    /// Window that the cursor left.
    pub entity: Entity,
}

/// An event that is sent whenever a window receives a character from the OS or underlying system.
#[derive(Debug, Clone)]
pub struct ReceivedCharacter {
    /// Window that received the character.
    pub entity: Entity,
    pub char: char,
}

/// An event that indicates a window has received or lost focus.
#[derive(Debug, Clone)]
pub struct WindowFocused {
    /// Window that changed focus.
    pub entity: Entity,
    /// Whether it was focused (true) or lost focused (false).
    pub focused: bool,
}

/// An event that indicates a window's scale factor has changed.
#[derive(Debug, Clone)]
pub struct WindowScaleFactorChanged {
    /// Window that had it's scale factor changed.
    pub entity: Entity,
    /// The new scale factor.
    pub scale_factor: f64,
}

/// An event that indicates a window's OS-reported scale factor has changed.
#[derive(Debug, Clone)]
pub struct WindowBackendScaleFactorChanged {
    pub entity: Entity,
    pub scale_factor: f64,
}

/// Events related to files being dragged and dropped on a window.
#[derive(Debug, Clone)]
pub enum FileDragAndDrop {
    DroppedFile {
        /// Window the file was dropped into.
        entity: Entity,
        /// Path to the file that was dropped in.
        path_buf: PathBuf,
    },

    HoveredFile {
        /// Window a file is possibly going to be dropped into.
        entity: Entity,
        /// Path to the file that might be dropped in.
        path_buf: PathBuf,
    },

    HoveredFileCancelled {
        /// Window that had a cancelled file drop.
        entity: Entity,
    },
}

/// An event that is sent when a window is repositioned in physical pixels.
#[derive(Debug, Clone)]
pub struct WindowMoved {
    /// Window that moved.
    pub entity: Entity,
    /// Where the window moved to in physical pixels.
    pub position: IVec2,
}
