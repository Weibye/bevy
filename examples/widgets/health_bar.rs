//! This example shows demonstrates how to setup a basic health bar with the [`StatusBarWidget`].

use core::time;
use std::thread;

use bevy::{
    math::map_range,
    prelude::*,
    sprite::MaterialMesh2dBundle,
    widget::{StatusBarInner, StatusBarWidget, WidgetPlugin},
};

/// Add a health component to an entity to track hit/health points.
#[derive(Component, Debug)]
struct Health {
    max: f32,
    hp: f32,
}

/// Label the player for queries.
#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WidgetPlugin)
        .add_startup_system(setup)
        .add_system(change_health)
        .add_system(set_status_bar) //.after(change_hp)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let health_bar_background: Color = Color::rgba_u8(54, 2, 2, 255);
    let health_bar_foreground: Color = Color::rgba_u8(42, 209, 56, 255);

    // spawn world camera
    commands.spawn(Camera2dBundle::default());

    // spawn player to take damage and heal (modified from [`2d_shapes`] example)
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes.add(shape::RegularPolygon::new(50., 6).into()).into(),
            material: materials.add(ColorMaterial::from(Color::GREEN)),
            ..default()
        })
        .insert((
            Health {
                max: 150.,
                hp: 150.,
            },
            Player,
        ));

    // spawn a status bar for health
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(35.0), Val::Percent(5.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(25.0),
                    bottom: Val::Px(35.),
                    ..default()
                },
                ..default()
            },
            background_color: health_bar_background.into(),
            ..default()
        })
        .insert(StatusBarWidget::new(0.0, 0., 1.))
        // spawn the moving inner bar
        .with_children(|outer| {
            outer
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: health_bar_foreground.into(),
                    ..default()
                })
                .insert(StatusBarInner);
        });
}

/// Update the [`StatusBarWidget`] with the current player health
fn set_status_bar(mut q: Query<&mut StatusBarWidget>, health: Query<&Health, With<Player>>) {
    for mut widget in q.iter_mut() {
        let health = health.single();
        let current_health = health.hp / health.max;
        widget.set_progress(current_health);
    }
}

/// Change player health using the arrow keys
fn change_health(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Health, With<Player>>) {
    let mut health = query.single_mut();
    if keyboard_input.just_pressed(KeyCode::Left) {
        health.hp -= 5.;
    } else if keyboard_input.just_pressed(KeyCode::Right) {
        health.hp += 5.;
    } else if keyboard_input.just_pressed(KeyCode::Space) {
        health.hp = health.max;
    }
}
