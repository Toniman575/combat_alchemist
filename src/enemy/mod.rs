use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{Attacking, GameLayer, Health, player::Player};

pub(super) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(enemy_attack)
            .add_systems(Update, (move_enemies, spawn_enemies))
            .add_systems(Startup, startup)
            .insert_resource(SpawnTimer(Timer::from_seconds(10., TimerMode::Repeating)));

        #[cfg(debug_assertions)]
        app.register_type::<Enemy>();
    }
}

#[derive(Resource, Reflect, DerefMut, Deref)]
struct SpawnTimer(Timer);

#[derive(Component, Reflect, Default)]
pub struct Moving;

#[derive(Component, Reflect)]
#[require(
    Health(30),
    Moving,
    RigidBody::Kinematic,
    Collider::circle(30.),
    TransformExtrapolation,
    CollisionLayers::new(GameLayer::Enemy, GameLayer::Player)
)]
pub struct Enemy {
    speed: f32,
}

impl Enemy {
    fn bundle(speed: f32, health: i32, collider_size: f32, name: String, pos: Vec2) -> impl Bundle {
        (
            Self { speed },
            Health(health),
            CollisionLayers::new(GameLayer::Enemy, GameLayer::Player),
            Collider::circle(collider_size),
            Name::new(name),
            Transform::from_translation(pos.extend(1.)),
        )
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(Enemy::bundle(
        150.,
        30,
        30.,
        String::from("Training Dummy"),
        Vec2::new(-100., -100.),
    ));
}

fn enemy_attack(
    trigger: Trigger<OnCollisionStart>,
    mut player: Single<(Entity, &mut Health), With<Player>>,
) {
    if player.0 == (trigger.collider) {
        player.1.0 -= 10;
    }
}

fn move_enemies(
    mut commands: Commands,
    enemy_q: Query<(Entity, &mut LinearVelocity, &Transform, &Enemy), With<Moving>>,
    player: Single<&Transform, With<Player>>,
) {
    for (enemy_entity, mut vel, enemy_transform, enemy) in enemy_q {
        let normalized_direction_vector =
            (player.translation.xy() - enemy_transform.translation.xy()).normalize_or_zero();

        if enemy_transform.translation.distance(player.translation) < 100. {
            let mut new_transform =
                Transform::from_translation((normalized_direction_vector * 80.).extend(0.));
            new_transform.rotation =
                Quat::from_rotation_arc(Vec3::Y, normalized_direction_vector.extend(0.));

            commands
                .entity(enemy_entity)
                .remove::<Moving>()
                .insert(Attacking {
                    target: new_transform,
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                });
            vel.set_if_neq(LinearVelocity::ZERO);
            continue;
        }

        vel.set_if_neq(LinearVelocity(normalized_direction_vector * enemy.speed));
    }
}

fn spawn_enemies(mut commands: Commands, mut timer: ResMut<SpawnTimer>, time: Res<Time>) {
    if timer.tick(time.delta()).finished() {
        commands.spawn(Enemy::bundle(
            150.,
            30,
            30.,
            String::from("Training Dummy"),
            Vec2::new(-100., -100.),
        ));
    }
}
