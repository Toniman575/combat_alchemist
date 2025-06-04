use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_cursor::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    AttackHitBoxTimer, GameLayer, Health,
    enemy::Enemy,
    player::input::{PrimaryAttack, SecondaryAttack},
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
    trigger: Trigger<Fired<PrimaryAttack>>,
    cursor_pos: Res<CursorLocation>,
    transform_q: Query<&Transform>,
    mut commands: Commands,
) {
    let Some(cursor_pos) = cursor_pos.world_position() else {
        return;
    };
    let player_transform = transform_q.get(trigger.target()).unwrap();
    let player_pos = player_transform.translation.xy();
    let direction_vector = cursor_pos - player_pos;

    let normalized_direction_vector = direction_vector.normalize_or_zero();

    let new_point = normalized_direction_vector * 100.;
    let mut new_transform = Transform::from_translation(new_point.extend(0.));
    new_transform.rotation =
        Quat::from_rotation_arc(Vec3::Y, normalized_direction_vector.extend(0.));

    commands.entity(trigger.target()).with_child((
        Collider::rectangle(5., 50.),
        Sensor,
        new_transform,
        CollisionEventsEnabled,
        AttackHitBoxTimer(Timer::from_seconds(0.1, TimerMode::Once)),
        CollisionLayers::new(GameLayer::PlayerAttack, GameLayer::Enemy),
    ));
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
