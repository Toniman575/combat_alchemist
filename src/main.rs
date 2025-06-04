mod camera;
mod enemy;
mod player;

use avian2d::{
    math::{AdjustPrecision, Scalar},
    prelude::*,
};
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_cursor::prelude::*;
use bevy_enhanced_input::prelude::*;
use rand::random;

#[cfg(debug_assertions)]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{
    camera::CameraPlugin,
    enemy::{EnemyPlugin, Moving},
    player::PlayerPlugin,
};

#[derive(InputContext)]
struct InGame;

#[derive(Component, Reflect)]
struct Attacking {
    target: Transform,
    timer: Timer,
}

#[derive(Component, Reflect)]
struct Health(i32);

#[derive(Component, Reflect, DerefMut, Deref)]
struct AttackHitBoxTimer(Timer);

#[derive(PhysicsLayer, Default)]
enum GameLayer {
    #[default]
    Default,
    Enemy,
    Player,
    EnemyAttack,
    PlayerAttack,
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
    .add_systems(Startup, startup)
    .add_systems(
        Update,
        (
            tick_hitbox_timer,
            tick_attack_timer,
            check_health,
            kinematic_collisions,
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
    .register_type::<Attacking>()
    .register_type::<Health>();

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
    time: Res<Time>,
) {
    for (entity, mut timer) in timer_q {
        if timer.tick(time.delta()).finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_attack_timer(
    mut commands: Commands,
    attacking_q: Query<(Entity, &mut Attacking)>,
    time: Res<Time>,
) {
    for (entity, mut attacking) in attacking_q {
        if attacking.timer.tick(time.delta()).finished() {
            commands.entity(entity).remove::<Attacking>().with_child((
                Collider::rectangle(5., 50.),
                Sensor,
                attacking.target,
                CollisionEventsEnabled,
                AttackHitBoxTimer(Timer::from_seconds(0.1, TimerMode::Once)),
                CollisionLayers::new(GameLayer::EnemyAttack, GameLayer::Player),
            ));
            commands.entity(entity).insert(Moving);
        }
    }
}

fn check_health(mut commands: Commands, health_q: Query<(Entity, &Health)>) {
    for (entity, health) in health_q {
        if health.0 < 0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Kinematic bodies do not get pushed by collisions by default,
/// so it needs to be done manually.
///
/// This system handles collision response for kinematic rigidbodies
/// by pushing them along their contact normals by the current penetration depth,
/// and applying velocity corrections in order to slide along walls, and other kinematic bodies
/// and predict collisions using speculative contacts.
#[allow(clippy::type_complexity)]
fn kinematic_collisions(
    collisions: Collisions,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut controllers: Query<(&mut Position, &mut LinearVelocity), With<RigidBody>>,
    time: Res<Time>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // Get the rigid body entities of the colliders (colliders could be children)
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
        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal_1 = -manifold.normal / 2.;
            let normal_2 = manifold.normal / 2.;
            let mut deepest_penetration: Scalar = Scalar::MIN;
            // Solve each penetrating contact in the manifold.
            for contact in manifold.points.iter() {
                if contact.penetration > 0.0 {
                    position_1.0 += normal_1 * contact.penetration;
                    position_2.0 += normal_2 * contact.penetration;
                }
                deepest_penetration = deepest_penetration.max(contact.penetration);
            }
            if deepest_penetration > 0. {
                // We want the character to slide along the surface, similarly to
                // a collide-and-slide algorithm.
                // Don't apply an impulse if the character is moving away from the surface.
                if linvel_1.dot(normal_1) < 0. {
                    // Slide along the surface, rejecting the velocity along the contact normal.
                    let impulse = linvel_1.reject_from_normalized(normal_1);
                    linvel_1.0 = impulse;
                }
                if linvel_2.dot(normal_2) < 0. {
                    // Slide along the surface, rejecting the velocity along the contact normal.
                    let impulse = linvel_2.reject_from_normalized(normal_2);
                    linvel_2.0 = impulse;
                }
            } else {
                // The character is not yet intersecting the other object,
                // but the narrow phase detected a speculative collision.
                //
                // We need to push back the part of the velocity
                // that would cause penetration within the next frame.
                let normal_speed_1 = linvel_1.dot(normal_1);
                let normal_speed_2 = linvel_1.dot(normal_2);
                // Don't apply an impulse if the character is moving away from the surface.
                if normal_speed_1 < 0.0 {
                    // Compute the impulse to apply.
                    let impulse_magnitude = normal_speed_1
                        - (deepest_penetration / time.delta_secs_f64().adjust_precision());
                    let mut impulse = impulse_magnitude * normal_1;
                    impulse.y = impulse.y.max(0.0);
                    linvel_1.0 -= impulse;
                }
                if normal_speed_2 < 0.0 {
                    // Compute the impulse to apply.
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
