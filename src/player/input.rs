use bevy::{
    color::palettes::css::{BLACK, WHITE},
    prelude::*,
};

use bevy_cursor::CursorLocation;
use bevy_enhanced_input::prelude::*;
use virtual_joystick::VirtualJoystickEvent;

use crate::{InGame, player::LookingDirection};

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub(super) struct PrimaryAttack;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub(super) struct SecondaryAttack;

#[derive(Component, Reflect)]
pub(crate) struct Button1;

#[derive(Component, Reflect)]
pub(crate) struct Button2;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub(crate) struct MovePlayer;

#[derive(Default, Debug, Reflect, Hash, Clone, PartialEq, Eq)]
pub(crate) enum JoystickID {
    LookingDirection,
    #[default]
    Movement,
}

pub(super) fn binding(trigger: Trigger<Binding<InGame>>, mut players: Query<&mut Actions<InGame>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();

    actions
        .bind::<MovePlayer>()
        .to(Cardinal::wasd_keys())
        .with_modifiers(DeadZone::default());

    actions
        .bind::<PrimaryAttack>()
        .to(Input::MouseButton {
            button: MouseButton::Left,
            mod_keys: ModKeys::empty(),
        })
        .with_conditions(Press::default());

    actions
        .bind::<SecondaryAttack>()
        .to(Input::MouseButton {
            button: MouseButton::Right,
            mod_keys: ModKeys::empty(),
        })
        .with_conditions(Press::default());
}

pub(super) fn update_looking_direction(
    cursor: Res<CursorLocation>,
    mut player: Single<(&mut LookingDirection, &Transform)>,
) {
    if let Some(cursor_pos) = cursor.world_position() {
        player.0.0 = (cursor_pos - player.1.translation.xy()).normalize();
    }
}

pub(super) fn update_joystick(
    mut commands: Commands,
    mut joystick: EventReader<VirtualJoystickEvent<JoystickID>>,
    mut player: Single<&mut LookingDirection>,
) {
    for joystick_events in joystick.read() {
        match joystick_events.id() {
            JoystickID::Movement => commands.trigger(Fired::<MovePlayer> {
                value: *joystick_events.axis(),
                state: ActionState::Fired,
                fired_secs: 0.,
                elapsed_secs: 0.,
            }),
            JoystickID::LookingDirection => {
                let Some(delta) = joystick_events.axis().try_normalize() else {
                    continue;
                };

                player.0 = delta;
            }
        }
    }
}

pub(super) fn button_system(
    mut commands: Commands,
    mut button_q: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            Has<Button1>,
            Has<Button2>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color, button_1, button_2) in &mut button_q {
        match *interaction {
            Interaction::Pressed => {
                if button_1 {
                    info!("button1 triggered");
                    commands.trigger(Fired::<PrimaryAttack> {
                        value: true,
                        state: ActionState::Fired,
                        fired_secs: 0.,
                        elapsed_secs: 0.,
                    });
                } else if button_2 {
                    info!("button2 triggered");
                    commands.trigger(Fired::<SecondaryAttack> {
                        value: true,
                        state: ActionState::Fired,
                        fired_secs: 0.,
                        elapsed_secs: 0.,
                    });
                }

                *color = BLACK.into();
                border_color.0 = WHITE.into();
            }
            Interaction::None => {
                *color = WHITE.into();
                border_color.0 = BLACK.into();
            }
            _ => {}
        }
    }
}
