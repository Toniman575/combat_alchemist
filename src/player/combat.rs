use std::time::Duration;

use avian2d::prelude::*;
use bevy::{prelude::*, time::Stopwatch};
use bevy_cursor::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    Attacking, Health, InGame, Moving,
    enemy::Enemy,
    player::{
        Player,
        input::{MovePlayer, PrimaryAttack, SecondaryAttack},
    },
};

#[derive(Component, Reflect)]
pub(super) struct Mark;

pub(super) fn apply_mark(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    enemy_q: Query<Entity, (With<Enemy>, Without<Mark>)>,
) {
    let Ok(enemy_entity) = enemy_q.get(trigger.collider) else {
        return;
    };

    commands.entity(enemy_entity).insert(Mark);
}

pub(super) fn primary_attack(
    _: Trigger<Fired<PrimaryAttack>>,
    cursor_pos: Res<CursorLocation>,
    player: Single<(Entity, &Transform, &Actions<InGame>), (With<Player>, Without<Attacking>)>,
    mut commands: Commands,
) {
    let Some(cursor_pos) = cursor_pos.world_position() else {
        return;
    };
    let (player_entity, player_transform, current_movement) = player.into_inner();
    let player_pos = player_transform.translation.xy();
    let direction_vector = cursor_pos - player_pos;

    let normalized_direction_vector = direction_vector.normalize_or_zero();

    let axis2d = current_movement.value::<MovePlayer>().unwrap().as_axis2d();
    commands
        .entity(player_entity)
        .remove::<Moving>()
        .insert(Attacking {
            target: normalized_direction_vector,
            rooted: Duration::from_secs_f32(0.35),
            spawn_hitbox: Duration::from_secs_f32(0.25),
            stopwatch: Stopwatch::new(),
            range: 100.,
            hitbox: Collider::rectangle(5., 50.),
            hitbox_duration: Duration::from_secs_f32(0.1),
            movement: Some(axis2d),
        });
}

pub(super) fn secondary_attack(
    _: Trigger<Fired<SecondaryAttack>>,
    mark_q: Query<(Entity, &mut Health), (With<Mark>, With<Enemy>)>,
    mut commands: Commands,
) {
    for (entity, mut health) in mark_q {
        health.0 -= 10;
        commands.entity(entity).remove::<Mark>();
    }
}
