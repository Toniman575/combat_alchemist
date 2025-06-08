use bevy::prelude::*;

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

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub(crate) struct MovePlayer;

#[derive(Default, Debug, Reflect, Hash, Clone, PartialEq, Eq)]
pub(crate) enum JoystickID {
    Button1,
    Button2,
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
            JoystickID::Button1 => {
                if let Some(delta) = joystick_events.axis().try_normalize() {
                    player.0 = delta;
                };

                if joystick_events.get_type() == virtual_joystick::VirtualJoystickEventType::Up {
                    commands.trigger(Fired::<PrimaryAttack> {
                        value: true,
                        state: ActionState::Fired,
                        fired_secs: 0.,
                        elapsed_secs: 0.,
                    })
                }
            }
            JoystickID::Button2 => {
                if let Some(delta) = joystick_events.axis().try_normalize() {
                    player.0 = delta;
                };

                if joystick_events.get_type() == virtual_joystick::VirtualJoystickEventType::Up {
                    commands.trigger(Fired::<SecondaryAttack> {
                        value: true,
                        state: ActionState::Fired,
                        fired_secs: 0.,
                        elapsed_secs: 0.,
                    })
                }
            }
        }
    }
}
