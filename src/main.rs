mod camera;
mod enemy;
mod player;

use avian2d::prelude::*;
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
struct HitBoxTimer(Timer);

#[derive(PhysicsLayer, Default)]
enum GameLayer {
    #[default]
    Default,
    Enemy,
    Player,
}

fn startup(mut commands: Commands) {
    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let custom_size = Some(Vec2::new(spacing, spacing));
    commands
        .spawn((Name::new("Map"), Transform::default(), Visibility::Inherited))
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
    timer_q: Query<(Entity, &mut HitBoxTimer)>,
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
                HitBoxTimer(Timer::from_seconds(0.1, TimerMode::Once)),
                CollisionLayers::new(GameLayer::Enemy, GameLayer::Player),
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
    .add_systems(Update, (tick_hitbox_timer, tick_attack_timer, check_health));

    #[cfg(debug_assertions)]
    app.add_plugins((
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        WorldInspectorPlugin::new(),
    ))
    .register_type::<HitBoxTimer>()
    .register_type::<Attacking>()
    .register_type::<Health>();

    app.run()
}
