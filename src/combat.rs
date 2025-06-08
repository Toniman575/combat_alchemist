use std::time::Duration;

use avian2d::prelude::{
    AngularVelocity, Collider, CollidingEntities, CollisionEventsEnabled, LinearVelocity,
    RigidBody, Sensor, TransformInterpolation,
};
use bevy::{prelude::*, time::Stopwatch};
use bevy_seedling::sample::{Sample, SamplePlayer};

use crate::{
    AppliesMark, AttackMarker, GameCollisionLayer, TriggersMark, ZLayer, audio::HitboxSound,
    enemy::Enemy, player::Player,
};

#[derive(Component)]
pub(super) struct Attacking {
    pub(super) hitbox: Vec<Collider>,
    pub(super) hitbox_duration: Vec<Duration>,
    pub(super) hitbox_movement: Vec<(LinearVelocity, AngularVelocity)>,
    pub(super) hitbox_sound: Vec<Handle<Sample>>,
    pub(super) marker: Option<AttackMarker>,
    pub(super) range: f32,
    pub(super) spawn_hitbox: Vec<Duration>,
    pub(super) sprite: Option<Sprite>,
    pub(super) stopwatch: Stopwatch,
    pub(super) swing_sound: Option<(Duration, Handle<Sample>)>,
    pub(super) target: Vec2,
}

#[derive(Component, Reflect)]
pub(super) struct AttackMovements {
    pub(super) movements: Vec<(Duration, AttackMovement)>,
    pub(super) stopwatch: Stopwatch,
}

#[derive(Reflect)]
pub(super) struct AttackMovement {
    pub(super) easing: EaseFunction,
    pub(super) duration: Duration,
    pub(super) from_to: (Vec2, Vec2),
    pub(super) speed: f32,
}
#[derive(Component, Reflect)]
pub(super) struct Swings {
    pub(super) swings: Vec<(Duration, Swing)>,
    pub(super) stopwatch: Stopwatch,
}

#[derive(Reflect)]
pub(super) struct Swing {
    pub(super) from: Transform,
    pub(super) to: Transform,
    pub(super) duration: Duration,
    pub(super) easing: EaseFunction,
}

#[derive(Component, Reflect)]
pub(super) struct Health {
    pub(super) current: i16,
    pub(super) max: i16,
}

#[derive(Component, Reflect)]
pub(crate) struct HealthBar;

#[derive(Component, Reflect, DerefMut, Deref)]
pub(super) struct AttackHitBoxTimer(pub(super) Timer);

pub(super) fn tick_hitbox_timer(
    mut commands: Commands,
    timer_q: Query<(Entity, &mut AttackHitBoxTimer)>,
    time: Res<Time<Virtual>>,
) {
    for (entity, mut timer) in timer_q {
        if timer.tick(time.delta()).finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn tick_attack_timer(
    mut commands: Commands,
    attacking_q: Query<(Entity, &mut Attacking, &Transform, Has<Enemy>, Has<Player>)>,
    time: Res<Time<Virtual>>,
) {
    for (entity, mut attacking, transform, is_enemy, is_player) in attacking_q {
        attacking.stopwatch.tick(time.delta());

        if let Some((duration, sound)) = &attacking.swing_sound
            && *duration <= attacking.stopwatch.elapsed()
        {
            commands.spawn(SamplePlayer::new(sound.clone_weak()));
            attacking.swing_sound = None;
        }

        let Some(spawn_hitbox_timer) = attacking.spawn_hitbox.last() else {
            commands.entity(entity).remove::<Attacking>();
            continue;
        };

        if *spawn_hitbox_timer <= attacking.stopwatch.elapsed() {
            attacking.spawn_hitbox.pop();

            let new_pos = attacking.target * attacking.range;
            let translation;

            let layer = if is_enemy {
                translation = new_pos.extend(ZLayer::EnemyWeapon.z_layer());
                GameCollisionLayer::enemy_attack()
            } else if is_player {
                translation = new_pos.extend(ZLayer::PlayerWeapon.z_layer());
                GameCollisionLayer::player_attack()
            } else if is_enemy && is_player {
                panic!("Entity is player and enemy?")
            } else {
                panic!("Entity is neither player nor enemy?")
            };

            let mut new_transform = Transform::from_translation(translation);

            if !attacking.hitbox_movement.is_empty() {
                new_transform.translation += transform.translation;
            }

            new_transform.rotation = Quat::from_rotation_arc(Vec3::Y, attacking.target.extend(0.));

            let mut child_entity_commands = commands.spawn_empty();

            child_entity_commands.insert((
                attacking.hitbox.pop().unwrap(),
                Sensor,
                new_transform,
                AttackHitBoxTimer(Timer::new(
                    attacking.hitbox_duration.pop().unwrap(),
                    TimerMode::Once,
                )),
                layer,
            ));

            if let Some(sprite) = &attacking.sprite {
                child_entity_commands.insert(sprite.clone());
            }

            if let Some(marker) = &attacking.marker {
                match marker {
                    AttackMarker::AppliesMark => child_entity_commands.insert(AppliesMark),
                    AttackMarker::TriggersMark => child_entity_commands.insert(TriggersMark),
                };
            }

            if let Some(handle) = &attacking.hitbox_sound.pop() {
                child_entity_commands.insert(HitboxSound(handle.clone_weak()));
            }

            if let Some(hitbox_movement) = attacking.hitbox_movement.pop() {
                child_entity_commands.insert((
                    TransformInterpolation,
                    hitbox_movement,
                    RigidBody::Kinematic,
                    CollidingEntities::default(),
                ));
            } else {
                child_entity_commands.insert(CollisionEventsEnabled);
                let child_entity = child_entity_commands.id();
                commands.entity(entity).add_child(child_entity);
            }
        }
    }
}

pub(super) fn attacking_movement(
    vel_q: Query<(Entity, &mut LinearVelocity, &mut AttackMovements)>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
) {
    let delta = time.delta();

    for (entity, mut lin_vel, mut attack_movement) in vel_q {
        attack_movement.stopwatch.tick(delta);
        let elapsed = attack_movement.stopwatch.elapsed();

        let Some((start, movement)) = attack_movement.movements.last() else {
            commands.entity(entity).remove::<AttackMovements>();
            continue;
        };

        if start >= &elapsed {
            continue;
        }

        if movement.duration >= elapsed {
            let t = (attack_movement.stopwatch.elapsed_secs() / movement.duration.as_secs_f32())
                .clamp(0., 1.);
            lin_vel.set_if_neq(LinearVelocity(
                movement.from_to.0.lerp(
                    movement.from_to.1,
                    EaseFunction::QuarticOut.sample(t).unwrap(),
                ) * movement.speed,
            ));
        } else {
            attack_movement.movements.pop();
        }
    }
}

pub(super) fn animate_swing(
    swing_q: Query<(Entity, &mut Transform, &mut Swings)>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
) {
    let delta = time.delta();

    for (entity, mut transform, mut swinging) in swing_q {
        swinging.stopwatch.tick(delta);

        let elapsed = swinging.stopwatch.elapsed();

        let Some((start, swing)) = swinging.swings.last() else {
            commands.entity(entity).remove::<Swings>();
            continue;
        };

        if start >= &elapsed {
            continue;
        }

        if swing.duration >= elapsed {
            let t = (elapsed.as_secs_f32() / swing.duration.as_secs_f32()).clamp(0., 1.);
            transform.translation = swing
                .from
                .translation
                .lerp(swing.to.translation, swing.easing.sample(t).unwrap());
        } else {
            swinging.swings.pop();
        }
    }
}
