//! This module contains basic node bundles used to build UIs

use crate::{
    widget::Button, BackgroundColor, CalculatedSize, FocusPolicy, Interaction, Node, Style,
    UiImage, ZIndex,
};
use bevy_ecs::bundle::Bundle;
use bevy_render::{
    prelude::{Color, ComputedVisibility},
    view::Visibility,
};
use bevy_text::{Text, TextAlignment, TextSection, TextStyle};
use bevy_transform::prelude::{GlobalTransform, Transform};

/// The basic UI node
///
/// Useful as a container for a variety of child nodes.
#[derive(Bundle, Clone, Debug)]
pub struct NodeBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Describes the style including flexbox settings
    pub style: Style,
    /// The background color, which serves as a "fill" for this node
    pub background_color: BackgroundColor,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// The transform of the node
    ///
    /// This field is automatically managed by the UI layout system.
    /// To alter the position of the `nodebundle`, use the properties of the [`Style`] component.
    pub transform: Transform,
    /// The global transform of the node
    ///
    /// This field is automatically managed by the UI layout system.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    /// Indicates the depth at which the node should appear in the UI
    pub z_index: ZIndex,
}

impl Default for NodeBundle {
    fn default() -> Self {
        NodeBundle {
            // Transparent background
            background_color: Color::NONE.into(),
            node: Default::default(),
            style: Default::default(),
            focus_policy: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
            z_index: Default::default(),
        }
    }
}

/// A UI node that is an image
#[derive(Bundle, Clone, Debug, Default)]
pub struct ImageBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Describes the style including flexbox settings
    pub style: Style,
    /// The calculated size based on the given image
    pub calculated_size: CalculatedSize,
    /// The background color, which serves as a "fill" for this node
    ///
    /// Combines with `UiImage` to tint the provided image.
    pub background_color: BackgroundColor,
    /// The image of the node
    pub image: UiImage,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// The transform of the node
    ///
    /// This field is automatically managed by the UI layout system.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub transform: Transform,
    /// The global transform of the node
    ///
    /// This field is automatically managed by the UI layout system.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    /// Indicates the depth at which the node should appear in the UI
    pub z_index: ZIndex,
}

/// A UI node that is text
#[derive(Bundle, Clone, Debug, Default)]
pub struct TextBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Describes the style including flexbox settings
    pub style: Style,
    /// Contains the text of the node
    pub text: Text,
    /// The calculated size based on the given image
    pub calculated_size: CalculatedSize,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// The transform of the node
    ///
    /// This field is automatically managed by the UI layout system.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub transform: Transform,
    /// The global transform of the node
    ///
    /// This field is automatically managed by the UI layout system.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    /// Indicates the depth at which the node should appear in the UI
    pub z_index: ZIndex,
}

/// A UI node that is a button
#[derive(Bundle, Clone, Debug)]
pub struct ButtonBundle {
    /// Describes the size of the node
    pub node: Node,
    /// Marker component that signals this node is a button
    pub button: Button,
    /// Describes the style including flexbox settings
    pub style: Style,
    /// Describes whether and how the button has been interacted with by the input
    pub interaction: Interaction,
    /// Whether this node should block interaction with lower nodes
    pub focus_policy: FocusPolicy,
    /// The background color, which serves as a "fill" for this node
    ///
    /// When combined with `UiImage`, tints the provided image.
    pub background_color: BackgroundColor,
    /// The image of the node
    pub image: UiImage,
    /// The transform of the node
    ///
    /// This field is automatically managed by the UI layout system.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub transform: Transform,
    /// The global transform of the node
    ///
    /// This field is automatically managed by the UI layout system.
    /// To alter the position of the `NodeBundle`, use the properties of the [`Style`] component.
    pub global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
    /// Indicates the depth at which the node should appear in the UI
    pub z_index: ZIndex,
}

impl Default for ButtonBundle {
    fn default() -> Self {
        Self {
            focus_policy: FocusPolicy::Block,
            node: Default::default(),
            button: Default::default(),
            style: Default::default(),
            interaction: Default::default(),
            background_color: Default::default(),
            image: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
            z_index: Default::default(),
        }
    }
}
