#[warn(missing_docs)]
mod cursor;
mod event;
mod raw_window_handle;
mod system;
mod window;

pub use crate::raw_window_handle::*;
use bevy_reflect::Reflect;
pub use cursor::*;
pub use event::*;
pub use system::*;
pub use window::*;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        CursorEntered, CursorIcon, CursorLeft, CursorMoved, FileDragAndDrop, ReceivedCharacter,
        Window, WindowBundle, WindowMoved,
    };
}

use bevy_app::prelude::*;
use bevy_ecs::{
    entity::Entity,
    event::Events,
    schedule::{ParallelSystemDescriptorCoercion, SystemLabel, SystemStage},
    system::{Command, Commands, ResMut, Resource, SystemState},
};

/// The configuration information for the [`WindowPlugin`].
///
/// It can be added as a [`Resource`](bevy_ecs::system::Resource) before the [`WindowPlugin`]
/// runs, to configure how it behaves.
#[derive(Resource, Clone)]
pub struct WindowSettings {
    /// Whether to create a window when added.
    ///
    /// Note that if there are no windows, by default the App will exit,
    /// due to [`exit_on_all_closed`].
    pub add_primary_window: bool,
    /// Whether to exit the app when there are no open windows.
    ///
    /// If disabling this, ensure that you send the [`bevy_app::AppExit`]
    /// event when the app should exit. If this does not occur, you will
    /// create 'headless' processes (processes without windows), which may
    /// surprise your users. It is recommended to leave this setting as `true`.
    ///
    /// If true, this plugin will add [`exit_on_all_closed`] to [`CoreStage::Update`].
    // TODO: Update documentation here
    pub exit_condition: ExitCondition,
    /// Whether to close windows when they are requested to be closed (i.e.
    /// when the close button is pressed).
    ///
    /// If true, this plugin will add [`close_when_requested`] to [`CoreStage::Update`].
    /// If this system (or a replacement) is not running, the close button will have no effect.
    /// This may surprise your users. It is recommended to leave this setting as `true`.
    pub close_when_requested: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        WindowSettings {
            add_primary_window: true,
            exit_condition: ExitCondition::OnAllClosed,
            close_when_requested: true,
        }
    }
}

/// A [`Plugin`] that defines an interface for windowing support in Bevy.
#[derive(Default)]
pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Window>()
            .register_type::<Cursor>()
            .register_type::<CursorPosition>()
            .register_type::<WindowResolution>()
            .register_type::<WindowPosition>()
            .register_type::<WindowMode>()
            .register_type::<PresentMode>()
            .register_type::<WindowTitle>()
            .register_type::<WindowState>()
            .register_type::<WindowCanvas>()
            .register_type::<Window>()
            .register_type::<WindowDecorations>()
            .register_type::<WindowTransparency>()
            .register_type::<WindowResizable>()
            .register_type::<WindowResizeConstraints>();

        app.add_event::<WindowResized>()
            .add_event::<WindowCreated>()
            .add_event::<WindowClosed>()
            .add_event::<WindowCloseRequested>()
            .add_event::<RequestRedraw>()
            .add_event::<CursorMoved>()
            .add_event::<CursorEntered>()
            .add_event::<CursorLeft>()
            .add_event::<ReceivedCharacter>()
            .add_event::<WindowFocused>()
            .add_event::<WindowScaleFactorChanged>()
            .add_event::<WindowBackendScaleFactorChanged>()
            .add_event::<FileDragAndDrop>()
            .add_event::<WindowMoved>();

        let settings = app
            .world
            .get_resource::<WindowSettings>()
            .cloned()
            .unwrap_or_default();

        if settings.add_primary_window {
            // TODO: Creating primary window should ideally be done through commands instead of the old way
            // however, commands aren't executed until the end of the "build-stage"
            // which means the primary-window does not exist until just before startup-systems starts running (?)
            // which means bevy_render does not have a window to use as attach to during plugin build.

            // Wishlist item; for this to work:
            // app.add_startup_system(create_primary_window);
            // or this:
            // app.add_build_system(create_primary_window)

            // TODO: The unwrap_or_default is necessary for the user to setup ahead of time what the window should be
            // if not we'll regress on this
            let window_bundle = app
                .world
                .remove_resource::<WindowBundle>()
                .unwrap_or_default();

            let window_id = app.world.spawn().insert_bundle(window_bundle).id();

            app.world
                .insert_resource(PrimaryWindow { window: window_id });
        }

        match settings.exit_condition {
            ExitCondition::OnPrimaryClosed => {
                app.add_system(exit_on_primary_closed);
            }
            ExitCondition::OnAllClosed => {
                app.add_system(exit_on_all_closed);
            }
            ExitCondition::DontExit => {}
        }

        if settings.close_when_requested {
            app.add_system_to_stage(CoreStage::First, close_when_requested);
        }
    }
}

/// System Label marking when changes are applied to windows
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub struct ModifiesWindows;

#[derive(Clone)]
pub enum ExitCondition {
    /// Close application when the primary window is closed
    OnPrimaryClosed,
    /// Close application when all windows are closed
    OnAllClosed,
    /// Keep application running headless even after closing all windows
    DontExit,
}

/// Resource containing the Entity that is currently considered the primary window.
///
/// This resource is allowed to not exist and should be handled gracefully if it doesn't.
#[derive(Debug, Resource, Clone, Reflect)]
pub struct PrimaryWindow {
    pub window: Entity,
}
