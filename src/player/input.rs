use bevy::prelude::*;

use bevy_enhanced_input::prelude::*;

use crate::InGame;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub(super) struct PrimaryAttack;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub(super) struct SecondaryAttack;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub(super) struct MovePlayer;

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
