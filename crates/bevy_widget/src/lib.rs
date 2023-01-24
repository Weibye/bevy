use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::schedule::IntoSystemDescriptor;
use bevy_ui::UiSystem;
use status_bar::update_status_bars;

mod status_bar;

pub use status_bar::*;

/// The basic plugin for Bevy Widget
#[derive(Default)]
pub struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<StatusBarWidget>()
            .register_type::<StatusBarInner>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                update_status_bars.before(UiSystem::Flex),
            );
    }
}
