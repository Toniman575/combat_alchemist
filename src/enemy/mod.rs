use std::time::Duration;

use avian2d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*, time::Stopwatch};

use crate::{Attacking, GameLayer, Health, HealthBar, Moving, player::Player};

pub(super) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(enemy_attack)
            .add_systems(Startup, startup)
            .add_systems(Update, (move_enemies, spawn_enemies))
            .insert_resource(SpawnTimer(Timer::from_seconds(10., TimerMode::Repeating)));

        #[cfg(debug_assertions)]
        app.register_type::<Enemy>();
    }
}

#[derive(Resource, Reflect, DerefMut, Deref)]
struct SpawnTimer(Timer);

#[derive(Component, Reflect)]
#[require(
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
    fn bundle(
        speed: f32,
        health: i16,
        collider_size: f32,
        name: String,
        pos: Vec2,
        mut meshes: ResMut<'_, Assets<Mesh>>,
        mut materials: ResMut<'_, Assets<ColorMaterial>>,
    ) -> impl Bundle {
        (
            Self { speed },
            Health {
                current: health,
                max: health,
            },
            CollisionLayers::new(
                GameLayer::Enemy,
                [[GameLayer::Enemy, GameLayer::Player, GameLayer::PlayerAttack]],
            ),
            Collider::circle(collider_size),
            Name::new(name),
            Transform::from_translation(pos.extend(1.)),
            children![(
                Mesh2d(meshes.add(Rectangle::new(32., 5.))),
                MeshMaterial2d(materials.add(Color::from(RED))),
                Transform::from_translation(Vec3::new(0., 50., 1.)),
                HealthBar,
                Name::new("Healthbar"),
                Visibility::Hidden,
            )],
        )
    }
}

fn startup(
    mut commands: Commands,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
) {
    commands.spawn(Enemy::bundle(
        150.,
        30,
        30.,
        String::from("Training Dummy"),
        Vec2::new(-100., -100.),
        meshes,
        materials,
    ));
}

fn enemy_attack(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    mut player: Single<(Entity, &mut Health), With<Player>>,
) {
    if player.0 == (trigger.collider) {
        player.1.current -= 10;
        commands.entity(trigger.target()).insert(ColliderDisabled);
    }
}

fn move_enemies(
    mut commands: Commands,
    enemy_q: Query<
        (Entity, &mut LinearVelocity, &Transform, &Enemy),
        (With<Moving>, Without<Attacking>),
    >,
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
                    target: normalized_direction_vector,
                    rooted: Duration::from_secs_f32(0.5),
                    spawn_hitbox: vec![Duration::from_secs_f32(0.4)],
                    stopwatch: Stopwatch::new(),
                    range: 80.,
                    hitbox: vec![Collider::rectangle(5., 50.)],
                    hitbox_duration: Duration::from_secs_f32(0.1),
                    movement: None,
                });
            vel.set_if_neq(LinearVelocity::ZERO);

            continue;
        }

        vel.set_if_neq(LinearVelocity(normalized_direction_vector * enemy.speed));
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut timer: ResMut<SpawnTimer>,
    time: Res<Time<Virtual>>,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
) {
    if timer.tick(time.delta()).finished() {
        commands.spawn(Enemy::bundle(
            150.,
            30,
            30.,
            String::from("Training Dummy"),
            Vec2::new(-100., -100.),
            meshes,
            materials,
        ));
    }
}
