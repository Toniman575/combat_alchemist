use std::time::Duration;

use avian2d::prelude::*;
use bevy::{platform::collections::HashSet, prelude::*, time::Stopwatch};
use bevy_enhanced_input::prelude::*;
use bevy_enoki::{ParticleEffectHandle, ParticleSpawner, prelude::OneShot};
use bevy_seedling::sample::SamplePlayer;

use crate::combat::{Attacking, Swing, Swings};
use crate::player::Moving;
use crate::{
    AttackMovements, AudioAssets, GameCollisionLayer, Health, InGame, ParticleAssets, Rooted,
    SpriteAssets, ZLayer,
    combat::AttackMovement,
    enemy::{Enemy, FollowedBy, Following},
    player::{
        LookingDirection, Player, WeaponSprite,
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
    enemy_q: Query<(Entity, &Position, &Rotation), (With<Enemy>, Without<Mark>)>,
    effect_assets: Res<ParticleAssets>,
    collisions: Collisions,
) {
    let Ok((enemy_entity, position, rotation)) = enemy_q.get(trigger.collider) else {
        return;
    };

    if !trigger_entity.contains(trigger.target()) {
        return;
    }

    commands.entity(enemy_entity).insert(Mark);

    let contact_point = &collisions
        .get(trigger.target(), trigger.body.unwrap())
        .unwrap()
        .manifolds
        .first()
        .unwrap()
        .find_deepest_contact()
        .unwrap()
        .global_point1(position, rotation);

    commands.spawn((
        ParticleSpawner::default(),
        ParticleEffectHandle(effect_assets.apply_mark.clone_weak()),
        OneShot::Despawn,
        Transform::from_translation(contact_point.extend(10.)),
    ));

    commands.spawn((
        Collider::circle(50.),
        Sensor,
        GameCollisionLayer::mark(),
        Transform::from_xyz(0., 0., 10.),
        Following::new(enemy_entity),
        Pickable::IGNORE,
        CollidingEntities::default(),
        ParticleSpawner::default(),
        ParticleEffectHandle(effect_assets.mark.clone_weak()),
    ));
}

pub(super) fn triggers_mark_collision(
    mut commands: Commands,
    colliding_q: Query<(Entity, &mut CollidingEntities), With<TriggersMark>>,
    enemy_q: Query<Option<&FollowedBy>, With<Enemy>>,
    audio_assets: Res<AudioAssets>,
) {
    for (entity, mut colliding_entites) in colliding_q {
        if colliding_entites.is_empty() {
            continue;
        }

        for colliding_entity in colliding_entites.drain() {
            let Ok(followed_by) = enemy_q.get(colliding_entity) else {
                continue;
            };

            if let Some(followed_by) = followed_by {
                for following_entity in followed_by.iter() {
                    let mut entities = HashSet::new();
                    entities.insert(following_entity);
                    commands
                        .entity(following_entity)
                        .trigger(TriggerMark(entities));
                    commands.entity(entity).despawn();
                    commands.spawn(SamplePlayer::new(audio_assets.mark_triggered.clone_weak()));
                }
                return;
            }
        }
        commands.entity(entity).despawn();
    }
}

pub(super) fn primary_attack(
    _: Trigger<Fired<PrimaryAttack>>,
    player: Single<(Entity, &Transform, &LookingDirection), (With<Player>, Without<Attacking>)>,
    player_weapon: Single<(Entity, &Transform), With<WeaponSprite>>,
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
) {
    let (player_entity, player_transform, direction_vector) = player.into_inner();
    let player_pos = player_transform.translation.xy();
    let normalized_direction_vector = direction_vector.normalize_or_zero();
    let rooted_duration = Duration::from_secs_f32(0.8);

    let mut binding = commands.entity(player_entity);
    let entity_commands = binding.remove::<Moving>().insert((
        Attacking {
            swing_sound: Some((
                Duration::from_secs_f32(0.1),
                audio_assets.staff_swing.clone_weak(),
            )),
            target: normalized_direction_vector,
            hitbox_movement: Vec::new(),
            spawn_hitbox: vec![Duration::from_secs_f32(0.25)],
            stopwatch: Stopwatch::new(),
            range: 20.,
            hitbox: vec![Collider::rectangle(4., 18.)],
            hitbox_duration: vec![Duration::from_secs_f32(0.1)],
            marker: Some(AttackMarker::AppliesMark),
            sprite: None,
            hitbox_sound: vec![audio_assets.staff_impact.clone_weak()],
        },
        Rooted {
            duration: rooted_duration,
            stopwatch: Stopwatch::new(),
        },
    ));

    entity_commands.insert(AttackMovements {
        movements: vec![
            (
                Duration::from_secs_f32(0.28),
                AttackMovement {
                    easing: EaseFunction::QuarticOut,
                    speed: 600.,
                    from_to: (-normalized_direction_vector, Vec2::ZERO),
                    duration: Duration::from_secs_f32(0.8),
                },
            ),
            (
                Duration::ZERO,
                AttackMovement {
                    easing: EaseFunction::QuarticOut,
                    speed: 300.,
                    from_to: (normalized_direction_vector, Vec2::ZERO),
                    duration: Duration::from_secs_f32(0.27),
                },
            ),
        ],
        stopwatch: Stopwatch::new(),
    });

    let mut transform = Transform::from_translation(
        (player_pos + normalized_direction_vector * 35.).extend(ZLayer::PlayerWeapon.z_layer()),
    );

    transform.rotation = Quat::from_rotation_arc(Vec3::Y, normalized_direction_vector.extend(0.));

    commands.entity(player_weapon.0).insert(Swings {
        swings: vec![(
            Duration::ZERO,
            Swing {
                from: *player_weapon.1,
                to: transform,
                duration: Duration::from_secs_f32(0.25),
                easing: EaseFunction::BackOut,
            },
        )],
        stopwatch: Stopwatch::new(),
    });
}

pub(super) fn secondary_attack(
    _: Trigger<Fired<SecondaryAttack>>,
    player: Single<
        (Entity, &Actions<InGame>, &LookingDirection),
        (With<Player>, Without<Attacking>),
    >,
    mut commands: Commands,
    sprite_assets: Res<SpriteAssets>,
) {
    let (player_entity, current_movement, direction_vector) = player.into_inner();

    let normalized_direction_vector = direction_vector.normalize_or_zero();

    let axis2d = current_movement.value::<MovePlayer>().unwrap().as_axis2d();
    let rooted_duration = Duration::from_secs_f32(0.25);
    commands.entity(player_entity).remove::<Moving>().insert((
        Attacking {
            swing_sound: None,
            target: normalized_direction_vector,
            hitbox_movement: vec![(
                LinearVelocity(normalized_direction_vector * 80.),
                AngularVelocity(15.),
            )],
            spawn_hitbox: vec![Duration::from_secs_f32(0.25)],
            stopwatch: Stopwatch::new(),
            range: 10.,
            hitbox: vec![Collider::circle(3.5)],
            hitbox_duration: vec![Duration::from_secs_f32(10.)],
            marker: Some(AttackMarker::TriggersMark),
            sprite: Some(Sprite {
                image: sprite_assets.potion.clone_weak(),
                custom_size: Some(Vec2::new(7., 7.)),
                ..default()
            }),
            hitbox_sound: Vec::new(),
        },
        AttackMovements {
            movements: vec![(
                Duration::ZERO,
                AttackMovement {
                    easing: EaseFunction::QuarticOut,
                    speed: 50.,
                    from_to: (axis2d, Vec2::ZERO),
                    duration: rooted_duration,
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
    mut query_mark: Query<(&Transform, &mut Health), With<Mark>>,
    mut commands: Commands,
    effect_assets: Res<ParticleAssets>,
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
    if let Ok((transform, mut health)) = query_mark.get_mut(trigger_entity_following) {
        health.current -= 10;
        let particle_transform =
            Transform::from_translation(transform.translation.truncate().extend(10.));
        commands.spawn((
            ParticleSpawner::default(),
            ParticleEffectHandle(effect_assets.trigger.clone_weak()),
            OneShot::Despawn,
            particle_transform,
        ));
    }

    commands.entity(trigger_entity).despawn();
}
