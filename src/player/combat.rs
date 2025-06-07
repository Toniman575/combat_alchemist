use std::time::Duration;

use avian2d::prelude::*;
use bevy::{platform::collections::HashSet, prelude::*, time::Stopwatch};
use bevy_cursor::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{
    AttackMovement, AttackMovements, Attacking, GameCollisionLayer, Health, InGame, Moving, Rooted,
    SpriteAssets, ZLayer,
    enemy::{Enemy, FollowedBy, Following},
    player::{
        Player, WeaponSprite,
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

#[derive(Component, Reflect)]
pub(super) struct Swinging {
    from: Transform,
    from_duration: Duration,
    from_easing: EaseFunction,
    stopwatch: Stopwatch,
    to: Transform,
    to_duration: Duration,
    to_easing: EaseFunction,
}

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
        Collider::circle(50.),
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
    player_weapon: Single<(Entity, &Transform), With<WeaponSprite>>,
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
    let rooted_duration = Duration::from_secs_f32(0.35);
    commands.entity(player_entity).remove::<Moving>().insert((
        Attacking {
            target: normalized_direction_vector,
            hitbox_movement: None,
            spawn_hitbox: vec![Duration::from_secs_f32(0.25)],
            stopwatch: Stopwatch::new(),
            range: 20.,
            hitbox: vec![Collider::rectangle(2., 15.)],
            hitbox_duration: Duration::from_secs_f32(0.1),
            marker: Some(AttackMarker::AppliesMark),
            sprite: None,
        },
        AttackMovements {
            movements: vec![(
                Duration::ZERO,
                AttackMovement {
                    easing: EaseFunction::QuarticOut,
                    speed: 50.,
                    from_to: (axis2d, Vec2::ZERO),
                    end_timing: rooted_duration,
                },
            )],
            stopwatch: Stopwatch::new(),
        },
        Rooted {
            duration: rooted_duration,
            stopwatch: Stopwatch::new(),
        },
    ));

    let mut transform = Transform::from_translation(
        (player_pos + normalized_direction_vector * 20.).extend(ZLayer::PlayerWeapon.z_layer()),
    );

    transform.rotation = Quat::from_rotation_arc(Vec3::Y, normalized_direction_vector.extend(0.));

    commands.entity(player_weapon.0).insert(Swinging {
        from: *player_weapon.1,
        from_easing: EaseFunction::BackIn,
        from_duration: Duration::from_secs_f32(0.25),
        to: transform,
        to_easing: EaseFunction::BackOut,
        to_duration: Duration::from_secs_f32(0.1),
        stopwatch: Stopwatch::new(),
    });
}

pub(super) fn secondary_attack(
    _: Trigger<Fired<SecondaryAttack>>,
    cursor_pos: Res<CursorLocation>,
    player: Single<(Entity, &Transform, &Actions<InGame>), (With<Player>, Without<Attacking>)>,
    mut commands: Commands,
    sprite_assets: Res<SpriteAssets>,
) {
    let Some(cursor_pos) = cursor_pos.world_position() else {
        return;
    };
    let (player_entity, player_transform, current_movement) = player.into_inner();
    let player_pos = player_transform.translation.xy();
    let direction_vector = cursor_pos - player_pos;

    let normalized_direction_vector = direction_vector.normalize_or_zero();

    let axis2d = current_movement.value::<MovePlayer>().unwrap().as_axis2d();
    let rooted_duration = Duration::from_secs_f32(0.35);
    commands.entity(player_entity).remove::<Moving>().insert((
        Attacking {
            target: normalized_direction_vector,
            hitbox_movement: Some(normalized_direction_vector),
            spawn_hitbox: vec![Duration::from_secs_f32(0.25)],
            stopwatch: Stopwatch::new(),
            range: 10.,
            hitbox: vec![Collider::circle(3.5)],
            hitbox_duration: Duration::from_secs_f32(10.),
            marker: Some(AttackMarker::TriggersMark),
            sprite: Some(Sprite {
                image: sprite_assets.potion.clone_weak(),
                custom_size: Some(Vec2::new(7., 7.)),
                ..default()
            }),
        },
        AttackMovements {
            movements: vec![(
                Duration::ZERO,
                AttackMovement {
                    easing: EaseFunction::QuarticOut,
                    speed: 50.,
                    from_to: (axis2d, Vec2::ZERO),
                    end_timing: rooted_duration,
                },
            )],
            stopwatch: Stopwatch::new(),
        },
        Rooted {
            duration: rooted_duration,
            stopwatch: Stopwatch::new(),
        },
    ));
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

    let Ok(collider) = colliders.get(trigger_entity) else {
        return;
    };
    let trigger_entity_following = collider.1.following();
    let Ok(mut binding) = colliding_entities.get_mut(trigger_entity) else {
        return;
    };
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

pub(super) fn animate_swing(
    swing_q: Query<(Entity, &mut Transform, &mut Swinging)>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
) {
    let delta = time.delta();

    for (entity, mut transform, mut swinging) in swing_q {
        swinging.stopwatch.tick(delta);
        let elapsed = swinging.stopwatch.elapsed();
        if swinging.from_duration >= swinging.stopwatch.elapsed() {
            let t = (elapsed.as_secs_f32() / swinging.from_duration.as_secs_f32()).clamp(0., 1.);
            transform.translation = transform.translation.lerp(
                swinging.to.translation,
                swinging.from_easing.sample(t).unwrap(),
            );
            transform.rotation = transform.rotation.lerp(
                swinging.from.rotation,
                swinging.from_easing.sample(t).unwrap(),
            );
        } else if swinging.to_duration >= swinging.stopwatch.elapsed() {
            let t = (elapsed.as_secs_f32() / swinging.to_duration.as_secs_f32()).clamp(0., 1.);
            transform.translation = transform.translation.lerp(
                swinging.from.translation,
                swinging.to_easing.sample(t).unwrap(),
            );
            transform.rotation = transform
                .rotation
                .lerp(swinging.to.rotation, swinging.to_easing.sample(t).unwrap());
        } else {
            commands.entity(entity).remove::<Swinging>();
        }
    }
}
