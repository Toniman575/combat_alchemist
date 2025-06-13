use std::time::Duration;

use avian2d::prelude::*;
use bevy::{prelude::*, time::Stopwatch};
use bevy_enhanced_input::prelude::*;
use bevy_enoki::{ParticleEffectHandle, ParticleSpawner, prelude::OneShot};
use bevy_seedling::sample::SamplePlayer;

use crate::Health;
use crate::combat::{Attacking, Swing, Swings};
use crate::player::Moving;
use crate::{
    AttackMovements, AudioAssets, GameCollisionLayer, InGame, ParticleAssets, Rooted, SpriteAssets,
    ZLayer,
    combat::AttackMovement,
    enemy::{Enemy, FollowedBy, Following},
    player::{
        LookingDirection, Player, WeaponSprite,
        input::{MovePlayer, PrimaryAttack, SecondaryAttack},
    },
};

#[derive(Component, Reflect)]
pub(super) struct Mark;

#[derive(Component, Reflect)]
pub(super) struct MarkTriggered;

#[derive(Component, Reflect, Copy, Clone)]
pub struct AppliesMark;

#[derive(Component, Reflect, Copy, Clone)]
pub struct TriggersMark;

pub enum AttackMarker {
    AppliesMark,
    TriggersMark,
}

#[derive(Event)]
pub struct TriggerMark;

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
        Transform::from_translation(contact_point.extend(ZLayer::Effects.z_layer())),
    ));

    commands.spawn((
        Collider::circle(50.),
        Sensor,
        GameCollisionLayer::mark(),
        Transform::from_xyz(0., 0., ZLayer::Effects.z_layer()),
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
    enemy_q: Query<Has<Mark>, With<Enemy>>,
    audio_assets: Res<AudioAssets>,
) {
    for (entity, mut colliding_entites) in colliding_q {
        if colliding_entites.is_empty() {
            continue;
        }

        for colliding_entity in colliding_entites.drain() {
            let Ok(has_mark) = enemy_q.get(colliding_entity) else {
                continue;
            };

            if has_mark {
                commands.entity(colliding_entity).trigger(TriggerMark);
                commands.entity(entity).despawn();
                commands.spawn(SamplePlayer::new(audio_assets.mark_triggered.clone_weak()));

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
            swings: None,
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
                    duration: Duration::from_secs_f32(1.2),
                },
            ),
            (
                Duration::ZERO,
                AttackMovement {
                    easing: EaseFunction::QuarticOut,
                    speed: 500.,
                    from_to: (normalized_direction_vector, Vec2::ZERO),
                    duration: Duration::from_secs_f32(0.27),
                },
            ),
        ],
        stopwatch: Stopwatch::new(),
    });

    let mut transform = Transform::from_translation(
        (player_pos + normalized_direction_vector * 40.).extend(ZLayer::PlayerWeapon.z_layer()),
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
            swings: None,
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
    mut collider_q: Query<&mut CollidingEntities, With<Sensor>>,
    mut enemy_q: Query<(Entity, &FollowedBy), With<Mark>>,
    mut commands: Commands,
) {
    let triggered_enemy = trigger.target();
    let (enemy_entity, followed_by) = enemy_q.get_mut(triggered_enemy).unwrap();

    let colliding_entites = collider_q
        .get_mut(followed_by.iter().last().unwrap())
        .unwrap();

    for entity in &colliding_entites.0 {
        let Ok((enemy_entity, _)) = enemy_q.get(*entity) else {
            continue;
        };

        let mut entity_commands = commands.entity(enemy_entity);
        entity_commands.remove::<Mark>();
        entity_commands.insert(MarkTriggered);
    }

    let mut entity_commands = commands.entity(enemy_entity);

    entity_commands.remove::<Mark>();
    entity_commands.insert(MarkTriggered);
}

pub(super) fn mark_triggered(
    mut commands: Commands,
    triggered_q: Query<(Entity, &Transform, &FollowedBy), With<MarkTriggered>>,
    mut colliding_entities: Query<&mut CollidingEntities>,
    mut health_q: Query<(&mut Health, Has<Mark>), With<Enemy>>,
    effect_assets: Res<ParticleAssets>,
) {
    for (entity, transform, followed_by) in triggered_q {
        for following_entity in followed_by.iter() {
            let mut colliding_entities = colliding_entities.get_mut(following_entity).unwrap();

            for colliding_entity in colliding_entities.drain() {
                let (mut health, has_mark) = health_q.get_mut(colliding_entity).unwrap();
                health.current -= 5;

                if has_mark {
                    let mut entity_commands = commands.entity(colliding_entity);
                    entity_commands.remove::<Mark>();
                    entity_commands.insert(MarkTriggered);
                }
            }
        }
        let (mut health, _) = health_q.get_mut(entity).unwrap();
        health.current -= 10;
        commands.entity(entity).remove::<MarkTriggered>();

        let particle_transform = Transform::from_translation(
            transform
                .translation
                .truncate()
                .extend(ZLayer::Effects.z_layer()),
        );
        commands.spawn((
            ParticleSpawner::default(),
            ParticleEffectHandle(effect_assets.trigger.clone_weak()),
            OneShot::Despawn,
            particle_transform,
        ));

        for following_entity in followed_by.iter() {
            commands.entity(following_entity).despawn();
        }
    }
}
