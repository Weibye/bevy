//! This example illustrates the various widgets in Bevy UI.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Icons::default())
        // Startup
        .add_startup_system(setup)
        .add_startup_system(load_icons)
        // Systems
        .add_system(button_system)
        .add_system(button_output)
        .add_system(toggle_system)
        .add_system(update_checkbox.after(toggle_system))
        .run();
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

const BUTTON_FONT_SIZE: f32 = 15.0;
const H1_FONT_SIZE: f32 = 30.0;


const FONT: &str = "fonts/FiraMono-Medium.ttf";
const CHECKBOX_EMPTY: &str = "textures/Icons/checkbox-empty.png";
const CHECKBOX_CHECKED: &str = "textures/Icons/checkbox-checked.png";

#[derive(Resource, Default)]
struct Icons {
    pub checkbox: Option<Handle<Image>>,
    pub checkbox_checked: Option<Handle<Image>>,
}

fn load_icons(mut icons: ResMut<Icons>, asset_server: Res<AssetServer>) {
    icons.checkbox = Some(asset_server.load(CHECKBOX_EMPTY));
    icons.checkbox_checked = Some(asset_server.load(CHECKBOX_CHECKED));
}

fn button_system(
    mut query: Query<(&Interaction, &mut UiColor, &Children),(Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, children) in &mut query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

/// System responsible for toggling the state of buttons that can toggle
fn toggle_system(
    mut q: Query<(&mut ToggleState, &Interaction), Changed<Interaction>>
) {
    for (mut state, interaction) in &mut q {
        match *interaction { 
            Interaction::Clicked => {
                state.0 = !state.0;
                info!("Toggled state to {:?}", state.0);
            }
            _ => { }
        }
    }
}

fn update_checkbox(
    mut q: Query<(&mut UiImage, &ToggleState), (Changed<ToggleState>, With<CheckBoxWidget>)>,
    icons: Res<Icons>,
) {
    for (mut image, state) in &mut q {
        image.0 = if state.0 {
            icons.checkbox_checked.unwrap()
        } else {
            icons.checkbox.unwrap()
        }
    }
}


fn button_output(q: Query<(Entity, &Interaction), Changed<Interaction>>) {
    for (entity, interaction) in &q {
        info!("Changed: {:?} : {:?}", entity, interaction);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // root node
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..default()
            },
            color: Color::rgb(0.9, 0.9, 0.9).into(),
            ..default()
        }).with_children(|root| {
            root.spawn_bundle(NodeBundle {
               style: Style {
                   size: Size::new(Val::Px(400.), Val::Auto),
                   flex_direction: FlexDirection::ColumnReverse,
                   margin: UiRect::all(Val::Px(5.)),
                   padding: UiRect::all(Val::Px(10.0)),
                   ..default()
               },
                color: Color::rgb(0.5, 0.5, 0.5).into(),
                ..default()
            }).with_children(|rect01 | {
                // Buttons title
                rect01.spawn_bundle(
                    TextBundle::from_section(
                        "Buttons", 
                        TextStyle {
                            font: asset_server.load(FONT),
                            font_size: H1_FONT_SIZE,
                            color: Color::WHITE
                        })
                );
                // Separator
                rect01.spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Px(1.0)),
                        margin: UiRect::new(Val::Undefined, Val::Px(5.0), Val::Undefined, Val::Px(5.0)),
                        ..default()
                    },
                    color: Color::WHITE.into(),
                    ..default()
                });
                
                // Button container
                rect01.spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Auto, Val::Px(45.0)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                }).with_children(|button_container| {
                    // Button 01
                    button_container.spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Auto, Val::Auto),
                            justify_content: JustifyContent::Center, // For centering button text
                            align_items: AlignItems::Center,         // For centering button text
                            flex_grow: 1.,
                            ..default()
                        },
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    }).with_children( | parent | {
                        parent.spawn_bundle(TextBundle::from_section(
                            "First button",
                            TextStyle {
                                font: asset_server.load(FONT),
                                font_size: BUTTON_FONT_SIZE,
                                color: Color::WHITE,
                            },
                        ));
                    });

                    // Button 02
                    button_container.spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Auto, Val::Auto),
                            // horizontally center child text
                            margin: UiRect::new(Val::Px(5.0), Val::Px(5.0), Val::Undefined, Val::Undefined),
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            flex_grow: 1.,
                            ..default()
                        },
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    }).with_children( | parent | {
                        parent.spawn_bundle(TextBundle::from_section(
                            "Second",
                            TextStyle {
                                font: asset_server.load(FONT),
                                font_size: BUTTON_FONT_SIZE,
                                color: Color::WHITE,
                            },
                        ));
                    });

                    // Button 03
                    button_container.spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Auto, Val::Auto),
                            // margin: UiRect::new(Val::Px(5.0), Val::Undefined, Val::Undefined, Val::Undefined),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            flex_grow: 1.,
                            ..default()
                        },
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    }).with_children( | parent | {
                        parent.spawn_bundle(TextBundle::from_section(
                            "Third",
                            TextStyle {
                                font: asset_server.load(FONT),
                                font_size: BUTTON_FONT_SIZE,
                                color: Color::WHITE,
                            },
                        ));
                    });
                });

                // Checkboxes title
                rect01.spawn_bundle(
                    TextBundle::from_section(
                        "Checkboxes",
                        TextStyle {
                            font: asset_server.load(FONT),
                            font_size: H1_FONT_SIZE,
                            color: Color::WHITE
                        })
                );
                // Separator
                rect01.spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Px(1.0)),
                        margin: UiRect::new(Val::Undefined, Val::Px(5.0), Val::Undefined, Val::Px(5.0)),
                        ..default()
                    },
                    color: Color::WHITE.into(),
                    ..default()
                });
        
                // Checkbox container
                rect01.spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Auto, Val::Px(45.0)),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                }).with_children(| container | {
                    container.spawn_bundle(ImageBundle {
                        style: Style {
                            // For some reason the aspect ratio works when this has _some_ size.
                            size: Size::new(Val::Px(1.), Val::Auto),
                            aspect_ratio: Some(1.0),
                            ..default()
                        },
                        color: Color::BLUE.into(),
                        image: asset_server.load(CHECKBOX_EMPTY).into(),
                        ..default()
                    })
                        .insert(Button)
                        .insert(Interaction::default())
                        .insert(CheckBoxWidget)
                        .insert(ToggleState(false));

                    container.spawn_bundle(ImageBundle {
                        style: Style {
                            // For some reason the aspect ratio works when this has _some_ size.
                            size: Size::new(Val::Px(1.), Val::Auto),
                            aspect_ratio: Some(1.0),
                            ..default()
                        },
                        color: Color::BLUE.into(),
                        image: asset_server.load(CHECKBOX_EMPTY).into(),
                        ..default()
                    })
                        .insert(Button)
                        .insert(Interaction::default())
                        .insert(CheckBoxWidget)
                        .insert(ToggleState(false));

                    container.spawn_bundle(ImageBundle {
                        style: Style {
                            // For some reason the aspect ratio works when this has _some_ size.
                            size: Size::new(Val::Px(1.), Val::Auto),
                            // TODO: We should support multiple aspect-ratio policies here:
                            // Height
                            aspect_ratio: Some(1.0),
                            ..default()
                        },
                        color: Color::BLUE.into(),
                        image: asset_server.load(CHECKBOX_EMPTY).into(),
                        ..default()
                    })
                        .insert(Button)
                        .insert(Interaction::default())
                        .insert(CheckBoxWidget)
                        .insert(ToggleState(false));
                    
                    container.spawn_bundle(ImageBundle {
                        style: Style {
                            // For some reason the aspect ratio works when this has _some_ size.
                            size: Size::new(Val::Px(1.), Val::Auto),
                            // TODO: We should support multiple aspect-ratio policies here:
                            // Height
                            aspect_ratio: Some(1.0),
                            ..default()
                        },
                        color: Color::BLUE.into(),
                        image: asset_server.load(CHECKBOX_EMPTY).into(),
                        ..default()
                    })
                        .insert(Button)
                        .insert(Interaction::default())
                        .insert(CheckBoxWidget)
                        .insert(ToggleState(false));
                    
                    container.spawn_bundle(ImageBundle {
                        style: Style {
                            // For some reason the aspect ratio works when this has _some_ size.
                            size: Size::new(Val::Px(1.), Val::Auto),
                            // TODO: We should support multiple aspect-ratio policies here:
                            // Height
                            aspect_ratio: Some(1.0),
                            ..default()
                        },
                        color: Color::BLUE.into(),
                        image: asset_server.load(CHECKBOX_EMPTY).into(),
                        ..default()
                    })
                        .insert(Button)
                        .insert(Interaction::default())
                        .insert(CheckBoxWidget)
                        .insert(ToggleState(false));
                });
            });
        });
}

/// Marker component for a CheckBoxWidget
#[derive(Component)]
struct CheckBoxWidget;

/// Stores the state of a toggled UI element
#[derive(Component)]
struct ToggleState(bool);