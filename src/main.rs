mod camera;
mod enemy;
mod player;
mod touch;

use std::time::Duration;

use avian2d::{
    math::{AdjustPrecision, Scalar},
    prelude::*,
};
use bevy::{asset::AssetMetaCheck, prelude::*, time::Stopwatch, window::WindowResolution};
use bevy_asset_loader::prelude::*;
use bevy_cursor::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_enoki::{EnokiPlugin, Particle2dEffect};
#[cfg(debug_assertions)]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_seedling::{
    SeedlingPlugin,
    sample::{Sample, SamplePlayer},
};
use rand::random;
use virtual_joystick::VirtualJoystickPlugin;

use crate::{
    camera::CameraPlugin,
    enemy::{Enemy, EnemyPlugin},
    player::{AppliesMark, AttackMarker, JoystickID, Player, PlayerPlugin, TriggersMark},
    touch::touch_interface,
};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AssetState {
    Loaded,
    #[default]
    Loading,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AssetState = AssetState::Loaded)]
enum GameState {
    Paused,
    #[default]
    Running,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
#[states(scoped_entities)]
enum CursorState {
    #[default]
    Mouse,
    Touch,
}

enum ZLayer {
    Enemies,
    EnemyWeapon,
    HealthBar,
    Map,
    Player,
    PlayerWeapon,
}

impl ZLayer {
    fn z_layer(&self) -> f32 {
        match self {
            ZLayer::Enemies => 2.,
            ZLayer::EnemyWeapon => 1.,
            ZLayer::HealthBar => 1.,
            ZLayer::Map => 0.,
            ZLayer::Player => 3.,
            ZLayer::PlayerWeapon => 3.5,
        }
    }
}

#[derive(AssetCollection, Resource)]
struct SpriteAssets {
    #[asset(path = "sprites/bite.png")]
    bite: Handle<Image>,
    #[asset(path = "sprites/enemy.png")]
    enemy: Handle<Image>,
    #[asset(path = "sprites/knob.png")]
    knob: Handle<Image>,
    #[asset(path = "sprites/outline.png")]
    outline: Handle<Image>,
    #[asset(path = "sprites/player.png")]
    player: Handle<Image>,

    #[asset(path = "sprites/potion.png")]
    potion: Handle<Image>,

    #[asset(path = "sprites/staff.png")]
    staff: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
struct ParticleAssets {
    #[asset(path = "effects/mark.ron")]
    mark: Handle<Particle2dEffect>,
    #[asset(path = "effects/trigger.ron")]
    trigger: Handle<Particle2dEffect>,
}

#[derive(AssetCollection, Resource)]
struct AudioAssets {
    #[asset(path = "audio/bite_impact.ogg")]
    bite_impact: Handle<Sample>,
    #[asset(path = "audio/bite_swing.ogg")]
    bite_swing: Handle<Sample>,
    #[asset(path = "audio/mark_triggered.ogg")]
    mark_triggered: Handle<Sample>,
    #[asset(path = "audio/staff_impact.ogg")]
    staff_impact: Handle<Sample>,
    #[asset(path = "audio/staff_swing.ogg")]
    staff_swing: Handle<Sample>,
}

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
    hitbox_movement: Option<Vec2>,
    hitbox_sound: Option<Handle<Sample>>,
    marker: Option<AttackMarker>,
    range: f32,
    spawn_hitbox: Vec<Duration>,
    sprite: Option<Sprite>,
    stopwatch: Stopwatch,
    swing_sound: Option<(Duration, Handle<Sample>)>,
    target: Vec2,
}

#[derive(Component, Reflect)]
struct Rooted {
    duration: Duration,
    stopwatch: Stopwatch,
}

#[derive(Component, Reflect)]
struct AttackMovements {
    movements: Vec<(Duration, AttackMovement)>,
    stopwatch: Stopwatch,
}

#[derive(Reflect)]
struct AttackMovement {
    easing: EaseFunction,
    end_timing: Duration,
    from_to: (Vec2, Vec2),
    speed: f32,
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

#[derive(Component, Reflect, DerefMut, Deref)]
struct HitboxSound(Handle<Sample>);

#[derive(PhysicsLayer, Default)]
enum GameCollisionLayer {
    #[default]
    Default,
    Enemy,
    EnemyAttack,
    Mark,
    Player,
    PlayerAttack,
}

impl GameCollisionLayer {
    fn enemy_attack() -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::EnemyAttack, GameCollisionLayer::Player)
    }

    fn mark() -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::Mark, GameCollisionLayer::Mark)
    }

    fn player_attack() -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::PlayerAttack, GameCollisionLayer::Enemy)
    }
}

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Window {
                    title: "Bevy New 2D".to_string(),
                    fit_canvas_to_parent: true,
                    resolution: WindowResolution::default().with_scale_factor_override(1.0),
                    ..default()
                }
                .into(),
                ..default()
            }),
        SeedlingPlugin::default(),
        EnokiPlugin,
        TrackCursorPlugin,
        EnhancedInputPlugin,
        VirtualJoystickPlugin::<JoystickID>::default(),
        PhysicsPlugins::default().with_length_unit(2.5),
        PhysicsPickingPlugin,
    ))
    .init_state::<AssetState>()
    .init_state::<CursorState>()
    .add_loading_state(
        LoadingState::new(AssetState::Loading)
            .continue_to_state(AssetState::Loaded)
            .load_collection::<SpriteAssets>()
            .load_collection::<ParticleAssets>()
            .load_collection::<AudioAssets>(),
    )
    .add_sub_state::<GameState>()
    // My plugins.
    .add_plugins((PlayerPlugin, EnemyPlugin, CameraPlugin))
    .add_input_context::<InGame>()
    .insert_resource(Gravity::ZERO)
    .add_observer(binding)
    .add_observer(pause_game)
    .add_observer(spawn_collision_sound)
    .add_systems(OnEnter(AssetState::Loaded), startup)
    .add_systems(OnEnter(CursorState::Touch), touch_interface)
    .add_systems(
        Update,
        (
            tick_hitbox_timer,
            update_healthbar,
            kinematic_collisions,
            tick_attack_timer,
            attacking_movement,
            tick_rooted,
            check_input_state,
        )
            .run_if(in_state(GameState::Running)),
    );

    #[cfg(debug_assertions)]
    app.add_plugins((
        PhysicsDebugPlugin::default(),
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        WorldInspectorPlugin::new(),
    ))
    .register_type::<AttackHitBoxTimer>()
    .register_type::<Health>()
    .register_type::<HealthBar>()
    .register_type::<AttackMovements>()
    .register_type::<Rooted>()
    .register_type::<HitboxSound>();

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
                        Transform::from_xyz(x, y, ZLayer::Map.z_layer()),
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

            if attacking.hitbox_movement.is_some() {
                new_transform.translation += transform.translation;
            }

            new_transform.rotation = Quat::from_rotation_arc(Vec3::Y, attacking.target.extend(0.));

            let mut child_entity_commands = commands.spawn_empty();

            child_entity_commands.insert((
                attacking.hitbox.pop().unwrap(),
                Sensor,
                new_transform,
                AttackHitBoxTimer(Timer::new(attacking.hitbox_duration, TimerMode::Once)),
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

            if let Some(handle) = &attacking.hitbox_sound {
                child_entity_commands.insert(HitboxSound(handle.clone_weak()));
            }

            if let Some(hitbox_movement) = attacking.hitbox_movement {
                child_entity_commands.insert((
                    TransformInterpolation,
                    LinearVelocity(hitbox_movement * 80.),
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

fn tick_rooted(
    rooted_q: Query<(Entity, &mut Rooted)>,
    mut commands: Commands,
    time: Res<Time<Virtual>>,
) {
    let delta = time.delta();
    for (entity, mut rooted) in rooted_q {
        rooted.stopwatch.tick(delta);

        if rooted.stopwatch.elapsed() >= rooted.duration {
            commands.entity(entity).remove::<Rooted>().insert(Moving);
        }
    }
}

fn attacking_movement(
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

        if movement.end_timing >= elapsed {
            let t = (attack_movement.stopwatch.elapsed_secs() / movement.end_timing.as_secs_f32())
                .clamp(0., 1.);
            lin_vel.set_if_neq(LinearVelocity(
                movement.from_to.0.lerp(
                    movement.from_to.1,
                    EaseFunction::QuarticOut.sample(t).unwrap(),
                ) * 50.,
            ));
        } else {
            attack_movement.movements.pop();
        }
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

fn pause_game(
    _: Trigger<Fired<Pause>>,
    mut time: ResMut<Time<Virtual>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if time.is_paused() {
        next_state.set(GameState::Running);
        time.unpause();
    } else {
        next_state.set(GameState::Paused);
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

fn check_input_state(
    cursor: Res<CursorLocation>,
    touch: Res<Touches>,
    mut next_input_state: ResMut<NextState<CursorState>>,
) {
    if cursor.get().is_some() {
        next_input_state.set(CursorState::Mouse);
    } else if touch.any_just_pressed() {
        next_input_state.set(CursorState::Touch)
    }
}

fn spawn_collision_sound(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    sound_q: Query<&HitboxSound>,
) {
    if let Ok(sound) = sound_q.get(trigger.target()) {
        commands.spawn(SamplePlayer::new(sound.0.clone_weak()));
    }
}
