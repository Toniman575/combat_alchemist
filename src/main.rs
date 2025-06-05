mod camera;
mod enemy;
mod player;

use std::time::Duration;

use avian2d::{
    math::{AdjustPrecision, Scalar},
    prelude::*,
};
use bevy::{asset::AssetMetaCheck, prelude::*, time::Stopwatch};
use bevy_cursor::prelude::*;
use bevy_enhanced_input::prelude::*;
use rand::random;

#[cfg(debug_assertions)]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{
    camera::CameraPlugin,
    enemy::{Enemy, EnemyPlugin},
    player::{Player, PlayerPlugin},
};

#[derive(InputContext)]
struct InGame;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Pause;

#[derive(Component, Reflect, Default)]
pub struct Moving;

#[derive(Component)]
struct Attacking {
    hitbox: Vec<Collider>,
    hitbox_duration: Duration,
    movement: Option<Vec2>,
    range: f32,
    rooted: Duration,
    spawn_hitbox: Vec<Duration>,
    stopwatch: Stopwatch,
    target: Vec2,
}

#[derive(Component, Reflect)]
struct Health {
    current: i16,
    max: i16,
}

#[derive(Component, Reflect)]
pub(crate) struct HealthBar;

#[derive(Component, Reflect, DerefMut, Deref)]
struct AttackHitBoxTimer(Timer);

#[derive(PhysicsLayer, Default)]
enum GameLayer {
    #[default]
    Default,
    Enemy,
    EnemyAttack,
    Player,
    PlayerAttack,
}

impl GameLayer {
    fn enemy_attack() -> CollisionLayers {
        CollisionLayers::new(GameLayer::EnemyAttack, GameLayer::Player)
    }

    fn player_attack() -> CollisionLayers {
        CollisionLayers::new(GameLayer::PlayerAttack, GameLayer::Enemy)
    }
}

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Window {
                    title: "Bevy New 2D".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
        TrackCursorPlugin,
        EnhancedInputPlugin,
        PhysicsDebugPlugin::default(),
        PhysicsPlugins::default().with_length_unit(100.),
    ))
    // My plugins.
    .add_plugins((PlayerPlugin, EnemyPlugin, CameraPlugin))
    .add_input_context::<InGame>()
    .insert_resource(Gravity::ZERO)
    .add_observer(binding)
    .add_observer(pause_game)
    .add_systems(Startup, startup)
    .add_systems(
        Update,
        (
            tick_hitbox_timer,
            update_healthbar,
            kinematic_collisions,
            tick_attack_timer,
            attacking_movement,
        ),
    );

    #[cfg(debug_assertions)]
    app.add_plugins((
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        WorldInspectorPlugin::new(),
    ))
    .register_type::<AttackHitBoxTimer>()
    .register_type::<Health>()
    .register_type::<HealthBar>();

    app.run()
}

fn startup(mut commands: Commands) {
    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let custom_size = Some(Vec2::new(spacing, spacing));
    commands
        .spawn((
            Name::new("Map"),
            Transform::default(),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            for x in 0..n {
                for y in 0..n {
                    let x = x as f32 * spacing - offset;
                    let y = y as f32 * spacing - offset;
                    let color = Color::hsl(240., random::<f32>() * 0.3, random::<f32>() * 0.3);
                    parent.spawn((
                        Sprite {
                            color,
                            custom_size,
                            ..default()
                        },
                        Transform::from_xyz(x, y, 0.),
                        Visibility::Inherited,
                    ));
                }
            }
        });
}

fn tick_hitbox_timer(
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

fn tick_attack_timer(
    mut commands: Commands,
    attacking_q: Query<(Entity, &mut Attacking, Has<Enemy>, Has<Player>)>,
    time: Res<Time<Virtual>>,
) {
    for (entity, mut attacking, is_enemy, is_player) in attacking_q {
        attacking.stopwatch.tick(time.delta());

        let mut remove_hitbox_timer = false;

        if let Some(spawn_hitbox_timer) = attacking.spawn_hitbox.last()
            && *spawn_hitbox_timer <= attacking.stopwatch.elapsed()
        {
            remove_hitbox_timer = true;

            let layer = if is_enemy {
                GameLayer::enemy_attack()
            } else if is_player {
                GameLayer::player_attack()
            } else if is_enemy && is_player {
                panic!("Entity is player and enemy?")
            } else {
                panic!("Entity is neither player nor enemy?")
            };

            let mut new_transform =
                Transform::from_translation((attacking.target * attacking.range).extend(2.));

            new_transform.rotation = Quat::from_rotation_arc(Vec3::Y, attacking.target.extend(0.));

            commands.entity(entity).with_child((
                attacking.hitbox.pop().unwrap(),
                Sensor,
                new_transform,
                CollisionEventsEnabled,
                AttackHitBoxTimer(Timer::new(attacking.hitbox_duration, TimerMode::Once)),
                layer,
            ));
        }

        if remove_hitbox_timer {
            attacking.spawn_hitbox.pop();
        }

        if attacking.rooted <= attacking.stopwatch.elapsed() {
            commands.entity(entity).remove::<Attacking>().insert(Moving);
        }
    }
}

fn attacking_movement(vel_q: Query<(&mut LinearVelocity, &Attacking)>) {
    for (mut lin_vel, attacking) in vel_q {
        let Some(movement) = attacking.movement else {
            lin_vel.set_if_neq(LinearVelocity::ZERO);
            continue;
        };

        let t = (attacking.stopwatch.elapsed_secs() / attacking.rooted.as_secs_f32()).clamp(0., 1.);
        lin_vel.set_if_neq(LinearVelocity(
            movement.lerp(Vec2::ZERO, EaseFunction::QuarticOut.sample(t).unwrap()) * 250.,
        ));
    }
}

#[allow(clippy::type_complexity)]
fn kinematic_collisions(
    collisions: Collisions,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut controllers: Query<(&mut Position, &mut LinearVelocity), With<RigidBody>>,
    time: Res<Time<Virtual>>,
) {
    for contacts in collisions.iter() {
        let Ok([&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }]) =
            collider_rbs.get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        let Ok(
            [
                (mut position_1, mut linvel_1),
                (mut position_2, mut linvel_2),
            ],
        ) = controllers.get_many_mut([rb1, rb2])
        else {
            continue;
        };

        for manifold in contacts.manifolds.iter() {
            let normal_1 = -manifold.normal / 2.;
            let normal_2 = manifold.normal / 2.;
            let mut deepest_penetration: Scalar = Scalar::MIN;

            for contact in manifold.points.iter() {
                if contact.penetration > 0.0 {
                    position_1.0 += normal_1 * contact.penetration;
                    position_2.0 += normal_2 * contact.penetration;
                }
                deepest_penetration = deepest_penetration.max(contact.penetration);
            }

            if deepest_penetration > 0. {
                if linvel_1.dot(normal_1) < 0. {
                    let impulse = linvel_1.reject_from_normalized(normal_1);
                    linvel_1.0 = impulse;
                }

                if linvel_2.dot(normal_2) < 0. {
                    let impulse = linvel_2.reject_from_normalized(normal_2);
                    linvel_2.0 = impulse;
                }
            } else {
                let normal_speed_1 = linvel_1.dot(normal_1);
                let normal_speed_2 = linvel_1.dot(normal_2);

                if normal_speed_1 < 0.0 {
                    let impulse_magnitude = normal_speed_1
                        - (deepest_penetration / time.delta_secs_f64().adjust_precision());
                    let mut impulse = impulse_magnitude * normal_1;
                    impulse.y = impulse.y.max(0.0);
                    linvel_1.0 -= impulse;
                }

                if normal_speed_2 < 0.0 {
                    let impulse_magnitude = normal_speed_2
                        - (deepest_penetration / time.delta_secs_f64().adjust_precision());
                    let mut impulse = impulse_magnitude * normal_2;
                    impulse.y = impulse.y.max(0.0);
                    linvel_2.0 -= impulse;
                }
            }
        }
    }
}

fn binding(trigger: Trigger<Binding<InGame>>, mut players: Query<&mut Actions<InGame>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Pause>()
        .to(Input::Keyboard {
            key: KeyCode::Space,
            mod_keys: ModKeys::empty(),
        })
        .with_conditions(Press::default());
}

fn pause_game(_: Trigger<Fired<Pause>>, mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
    } else {
        time.pause();
    }
}

pub(crate) fn update_healthbar(
    mut commands: Commands,
    changed: Query<'_, '_, (Entity, &Health, &Children, Option<&Enemy>), Changed<Health>>,
    mut transforms: Query<'_, '_, (&mut Visibility, &mut Transform), With<HealthBar>>,
) {
    for (entity, health, children, enemy) in &changed {
        if health.current <= 0 {
            commands.entity(entity).despawn();
            continue;
        }

        for child in children {
            if let Ok((mut visibility, mut transform)) = transforms.get_mut(*child) {
                let percentage = f32::from(health.current) / f32::from(health.max);
                transform.scale.x = percentage;
                transform.translation.x = -16. * (1. - percentage);

                if enemy.is_some() {
                    if percentage < 1. {
                        visibility.set_if_neq(Visibility::Visible);
                    } else {
                        visibility.set_if_neq(Visibility::Hidden);
                    }
                }
            }
        }
    }
}
