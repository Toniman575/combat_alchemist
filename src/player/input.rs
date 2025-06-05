use bevy::prelude::*;

use bevy_enhanced_input::prelude::*;

use crate::{InGame, enemy::Enemy};

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub(super) struct PrimaryAttack;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub(super) struct SecondaryAttack;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub(super) struct MovePlayer;

#[derive(Component, Reflect)]
pub(super) struct Mouseover;

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

pub(super) fn add_mouseover(
    trigger: Trigger<Pointer<Over>>,
    mut commands: Commands<'_, '_>,
    enemy_q: Query<Entity, With<Enemy>>,
) {
    if enemy_q.contains(trigger.target) {
        commands.entity(trigger.target).insert(Mouseover);
    }
}

pub(super) fn remove_mouseover(trigger: Trigger<Pointer<Out>>, mut commands: Commands<'_, '_>) {
    commands.entity(trigger.target).remove::<Mouseover>();
}
