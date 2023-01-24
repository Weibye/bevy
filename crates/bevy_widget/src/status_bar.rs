//! A status bar widget.
//! Can be used for loading bars, but also health-bars, mana, those kind of things.

use core::panic;

use bevy_ecs::{
    prelude::Component,
    query::{Changed, With},
    reflect::ReflectComponent,
    system::Query,
};
use bevy_hierarchy::Children;
use bevy_log::warn;
use bevy_math::map_range;
use bevy_reflect::Reflect;
use bevy_ui::{Size, Style, Val};

/// A status bar widget.
#[derive(Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct StatusBarWidget {
    /// The current progress of the progress bar.
    ///
    /// Valid range between min and max, inclusive.
    progress: f32,
    /// Minimum valid value that progress can have. Inclusive.
    min: f32,
    /// Maximum valid value that progress can have. Inclusive.
    max: f32,
    /// Defines the direction of the `ProgressBarWidget`.
    direction: StatusBarDirection,
}

/// Defines the direction the progress bar will increase the size of the inner node.
///
/// It increases in the direction of the flex-axis.
#[derive(Default, Debug, Clone, Reflect)]
pub enum StatusBarDirection {
    /// Direction from FlexStart to FlexEnd
    #[default]
    Horizontal,
    /// Direction from CrossStart to CrossEnd
    Vertical,
}

/// Marker component for the inner box of the progress bar.
#[derive(Component, Default, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct StatusBarInner;

impl StatusBarWidget {
    /// Creates a new [`StatusBarWidget`].
    pub fn new(progress: f32, min: f32, max: f32) -> Self {
        if min > max {
            panic!("Min should not be larger than max");
        } else {
            StatusBarWidget {
                progress,
                min,
                max,
                direction: StatusBarDirection::default(),
            }
        }
    }

    /// Gets the current progress.
    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    /// Sets the current progress.
    ///
    /// Will output warning if trying to set a value outside the valid range.
    // TODO: allow this to handle overflow for health and other non-loading cases
    pub fn set_progress(&mut self, progress: f32) {
        if progress >= self.min && progress <= self.max {
            self.progress = progress;
        } else {
            match progress {
                i if i < self.min => self.progress = 0.,
                // i if i > self.min && i < self.max => self.progress = progress,
                i if i > self.max => self.progress = 1.,
                _ => panic!("outside of range"),
            };
        }
    }
}

pub(crate) fn update_status_bars(
    q: Query<(&StatusBarWidget, &Children), Changed<StatusBarWidget>>,
    mut inner: Query<&mut Style, With<StatusBarInner>>,
) {
    for (widget, children) in q.iter() {
        for child in children.iter() {
            if let Ok(mut style) = inner.get_mut(*child) {
                let current_size = style.size;
                let new_value = Val::Percent(map_range(
                    widget.get_progress(),
                    (widget.min, widget.max),
                    (0., 100.0),
                ));

                style.size = match widget.direction {
                    StatusBarDirection::Horizontal => Size::new(new_value, current_size.height),
                    StatusBarDirection::Vertical => Size::new(current_size.width, new_value),
                };
            }
        }
    }
}
