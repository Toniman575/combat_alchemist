use bevy::prelude::*;
use virtual_joystick::{
    JoystickFloating, NoAction, VirtualJoystickBundle, VirtualJoystickNode,
    VirtualJoystickUIBackground, VirtualJoystickUIKnob,
};

use crate::{CursorState, JoystickID, SpriteAssets};

pub(super) fn touch_interface(mut commands: Commands, sprite_assets: Res<SpriteAssets>) {
    commands
        .spawn((
            VirtualJoystickBundle::new(
                VirtualJoystickNode::<JoystickID>::default()
                    .with_id(JoystickID::Movement)
                    .with_behavior(JoystickFloating)
                    .with_action(NoAction),
            )
            .set_style(Node {
                width: Val::Px(150.),
                height: Val::Px(150.),
                position_type: PositionType::Absolute,
                left: Val::Px(150.),
                bottom: Val::Px(150.),
                ..default()
            }),
            StateScoped(CursorState::Touch),
        ))
        .with_children(|parent| {
            parent.spawn((
                VirtualJoystickUIKnob,
                ImageNode {
                    color: Color::WHITE.with_alpha(1.0),
                    image: sprite_assets.knob.clone_weak(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(Vec2::new(75., 75.).x),
                    height: Val::Px(Vec2::new(75., 75.).y),
                    ..default()
                },
                ZIndex(1),
            ));

            parent.spawn((
                VirtualJoystickUIBackground,
                ImageNode {
                    color: Color::WHITE.with_alpha(1.0),
                    image: sprite_assets.outline.clone_weak(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(Vec2::new(150., 150.).x),
                    height: Val::Px(Vec2::new(150., 150.).y),
                    ..default()
                },
                ZIndex(0),
            ));
        });

    commands
        .spawn((
            VirtualJoystickBundle::new(
                VirtualJoystickNode::<JoystickID>::default()
                    .with_id(JoystickID::Button1)
                    .with_behavior(JoystickFloating)
                    .with_action(NoAction),
            )
            .set_style(Node {
                width: Val::Px(150.),
                height: Val::Px(150.),
                position_type: PositionType::Absolute,
                right: Val::Px(150.),
                bottom: Val::Px(400.),
                ..default()
            }),
            StateScoped(CursorState::Touch),
        ))
        .with_children(|parent| {
            parent.spawn((
                VirtualJoystickUIKnob,
                ImageNode {
                    color: Color::WHITE.with_alpha(1.0),
                    image: sprite_assets.knob.clone_weak(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(Vec2::new(75., 75.).x),
                    height: Val::Px(Vec2::new(75., 75.).y),
                    ..default()
                },
                ZIndex(1),
            ));

            parent.spawn((
                VirtualJoystickUIBackground,
                ImageNode {
                    color: Color::WHITE.with_alpha(1.0),
                    image: sprite_assets.outline.clone_weak(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(Vec2::new(150., 150.).x),
                    height: Val::Px(Vec2::new(150., 150.).y),
                    ..default()
                },
                ZIndex(0),
            ));
        });

    commands
        .spawn((
            VirtualJoystickBundle::new(
                VirtualJoystickNode::<JoystickID>::default()
                    .with_id(JoystickID::Button2)
                    .with_behavior(JoystickFloating)
                    .with_action(NoAction),
            )
            .set_style(Node {
                width: Val::Px(150.),
                height: Val::Px(150.),
                position_type: PositionType::Absolute,
                right: Val::Px(400.),
                bottom: Val::Px(150.),
                ..default()
            }),
            StateScoped(CursorState::Touch),
        ))
        .with_children(|parent| {
            parent.spawn((
                VirtualJoystickUIKnob,
                ImageNode {
                    color: Color::WHITE.with_alpha(1.0),
                    image: sprite_assets.knob.clone_weak(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(Vec2::new(75., 75.).x),
                    height: Val::Px(Vec2::new(75., 75.).y),
                    ..default()
                },
                ZIndex(1),
            ));

            parent.spawn((
                VirtualJoystickUIBackground,
                ImageNode {
                    color: Color::WHITE.with_alpha(1.0),
                    image: sprite_assets.outline.clone_weak(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(Vec2::new(150., 150.).x),
                    height: Val::Px(Vec2::new(150., 150.).y),
                    ..default()
                },
                ZIndex(0),
            ));
        });
}
