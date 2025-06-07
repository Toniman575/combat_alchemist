use bevy::{color::palettes::css::WHITE, prelude::*};
use virtual_joystick::{
    JoystickFloating, NoAction, VirtualJoystickBundle, VirtualJoystickNode,
    VirtualJoystickUIBackground, VirtualJoystickUIKnob,
};

use crate::{
    CursorState, JoystickID, SpriteAssets,
    player::{Button1, Button2},
};

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
                width: Val::Px(75.),
                height: Val::Px(75.),
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
                    .with_id(JoystickID::LookingDirection)
                    .with_behavior(JoystickFloating)
                    .with_action(NoAction),
            )
            .set_style(Node {
                width: Val::Px(75.),
                height: Val::Px(75.),
                position_type: PositionType::Absolute,
                right: Val::Px(150.),
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

    commands.spawn((
        Node {
            width: Val::Px(100.),
            height: Val::Px(100.),
            position_type: PositionType::Absolute,
            right: Val::Px(175.),
            bottom: Val::Px(325.),
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(Color::Srgba(WHITE)),
            Button1,
        )],
        StateScoped(CursorState::Touch),
    ));

    commands.spawn((
        Node {
            width: Val::Px(100.),
            height: Val::Px(100.),
            position_type: PositionType::Absolute,
            right: Val::Px(325.),
            bottom: Val::Px(175.),
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(Color::Srgba(WHITE)),
            Button2,
        )],
        StateScoped(CursorState::Touch),
    ));
}
