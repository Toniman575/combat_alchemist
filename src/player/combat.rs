use std::time::Duration;

use avian2d::prelude::*;
use bevy::{platform::collections::HashSet, prelude::*, time::Stopwatch};
use bevy_cursor::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    Attacking, GameCollisionLayer, Health, InGame, Moving,
    enemy::{Enemy, FollowedBy, Following},
    player::{
        Player,
        input::{MovePlayer, PrimaryAttack, SecondaryAttack},
    },
};

#[derive(Component, Reflect)]
pub(super) struct Mark;

#[derive(Component, Reflect, Copy, Clone)]
pub struct AppliesMark;

#[derive(Component, Reflect, Copy, Clone)]
pub struct TriggersMark;

pub enum AttackMarker {
    AppliesMark,
    TriggersMark,
}

#[derive(Event)]
pub struct TriggerMark(HashSet<Entity>);

pub(super) fn apply_mark(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    trigger_entity: Query<Entity, With<AppliesMark>>,
    enemy_q: Query<Entity, (With<Enemy>, Without<Mark>)>,
) {
    let Ok(enemy_entity) = enemy_q.get(trigger.collider) else {
        return;
    };

    if !trigger_entity.contains(trigger.target()) {
        return;
    }

    commands.entity(enemy_entity).insert(Mark);

    commands.spawn((
        Collider::circle(100.),
        Sensor,
        GameCollisionLayer::mark(),
        Transform::from_xyz(0., 0., 0.),
        Following::new(enemy_entity),
        Pickable::IGNORE,
        CollidingEntities::default(),
    ));
}

pub(super) fn triggers_mark_collision(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    trigger_entity: Query<Entity, With<TriggersMark>>,
    enemy_q: Query<Option<&FollowedBy>, With<Enemy>>,
) {
    let Ok(has_mark) = enemy_q.get(trigger.collider) else {
        return;
    };

    let target_entity = trigger.target();

    if !trigger_entity.contains(target_entity) {
        return;
    }

    if let Some(followed_by) = has_mark {
        for entity in followed_by.iter() {
            let mut entities = HashSet::new();
            entities.insert(entity);
            commands.entity(entity).trigger(TriggerMark(entities));
            commands.entity(target_entity).despawn();
        }
    } else {
        commands.entity(target_entity).despawn();
    }
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
            hitbox_movement: None,
            rooted: Duration::from_secs_f32(0.35),
            spawn_hitbox: vec![Duration::from_secs_f32(0.25)],
            stopwatch: Stopwatch::new(),
            range: 100.,
            hitbox: vec![Collider::rectangle(5., 50.)],
            hitbox_duration: Duration::from_secs_f32(0.1),
            movement: Some(axis2d),
            marker: Some(AttackMarker::AppliesMark),
        });
}

pub(super) fn secondary_attack(
    _: Trigger<Fired<SecondaryAttack>>,
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
            hitbox_movement: Some(normalized_direction_vector),
            rooted: Duration::from_secs_f32(0.35),
            spawn_hitbox: vec![Duration::from_secs_f32(0.25)],
            stopwatch: Stopwatch::new(),
            range: 30.,
            hitbox: vec![Collider::circle(5.)],
            hitbox_duration: Duration::from_secs_f32(10.),
            movement: Some(axis2d),
            marker: Some(AttackMarker::TriggersMark),
        });
}

pub(super) fn trigger_mark(
    trigger: Trigger<TriggerMark>,
    mut colliding_entities: Query<&mut CollidingEntities>,
    colliders: Query<(Entity, &Following), With<Sensor>>,
    mut query_mark: Query<&mut Health, With<Mark>>,
    mut commands: Commands,
) {
    let trigger_entity = trigger.target();
    if commands.get_entity(trigger_entity).is_err() {
        return;
    }

    let trigger_entity_following = colliders.get(trigger_entity).unwrap().1.following();
    let mut binding = colliding_entities.get_mut(trigger_entity).unwrap();
    let mut already_triggered_entities = trigger.0.clone();
    let mut entities_being_triggered_now = binding.clone();
    already_triggered_entities.extend(entities_being_triggered_now.drain());

    for colliding_entity in binding.drain() {
        if trigger.0.contains(&colliding_entity) {
            continue;
        }

        let Ok((collider_entity, _)) = colliders.get(colliding_entity) else {
            continue;
        };

        commands
            .entity(collider_entity)
            .trigger(TriggerMark(already_triggered_entities.clone()));
    }

    commands.entity(trigger_entity_following).remove::<Mark>();
    query_mark
        .get_mut(trigger_entity_following)
        .unwrap()
        .current -= 10;
    commands.entity(trigger_entity).despawn();
}
