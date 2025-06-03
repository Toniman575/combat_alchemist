use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_cursor::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{EguiContexts, EguiPlugin},
    quick::WorldInspectorPlugin,
};
use rand::random;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Zoom;

#[derive(InputContext)]
struct InGame;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct PrimaryAttack;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct SecondaryAttack;

#[derive(Component, Reflect)]
#[require(Health(100))]
struct Player {
    speed: f32,
}

impl Player {
    fn new(speed: f32) -> Self {
        Self { speed }
    }
}

#[derive(Component, Reflect)]
#[require(Health(30), Moving)]
struct Enemy {
    speed: f32,
}

impl Enemy {
    fn new(speed: f32) -> Self {
        Self { speed }
    }
}

#[derive(Component, Reflect, Default)]
struct Moving;

#[derive(Component, Reflect)]
struct Mark;

#[derive(Component, Reflect)]
struct Attacking {
    target: Transform,
    timer: Timer,
}

#[derive(Component, Reflect)]
struct Dead;

#[derive(Component, Reflect)]
struct Health(i32);

#[derive(Component, Reflect, DerefMut, Deref)]
struct HitBoxTimer(Timer);

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Window {
                    title: String::from("Combat Alchemist"),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
            TrackCursorPlugin,
            EnhancedInputPlugin,
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            PhysicsDebugPlugin::default(),
            PhysicsPlugins::default().with_length_unit(100.),
            WorldInspectorPlugin::new(),
        ))
        .add_input_context::<InGame>()
        .add_observer(apply_velocity)
        .add_observer(stop_velocity)
        .add_observer(binding)
        .add_observer(zoom)
        .add_observer(primary_attack)
        .add_observer(secondary_attack)
        .add_observer(apply_mark)
        .add_observer(enemy_attack)
        .insert_resource(Gravity::ZERO)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                update_camera,
                tick_hitbox_timer,
                tick_attack_timer,
                move_enemies,
                check_health,
            ),
        )
        .register_type::<Player>()
        .register_type::<Enemy>()
        .register_type::<HitBoxTimer>()
        .register_type::<Attacking>()
        .register_type::<Mark>()
        .register_type::<Health>()
        .register_type::<Attacking>()
        .register_type::<Dead>()
        .run()
}

fn apply_mark(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    enemy_q: Query<Entity, (With<Enemy>, Without<Mark>, Without<Dead>)>,
) {
    let Ok(enemy_entity) = enemy_q.get(trigger.collider) else {
        return;
    };

    commands.entity(enemy_entity).insert(Mark);
}

fn enemy_attack(
    trigger: Trigger<OnCollisionStart>,
    mut player: Single<(Entity, &mut Health), With<Player>>,
) {
    if player.0 == (trigger.collider) {
        player.1.0 -= 10;
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Name::new("Player"),
        Player::new(250.),
        RigidBody::Kinematic,
        Collider::circle(25.),
        TransformExtrapolation,
        Actions::<InGame>::default(),
        Transform::from_xyz(0., 0., 1.),
    ));

    commands.spawn((
        Name::new("Training Dummy"),
        Enemy::new(150.),
        RigidBody::Kinematic,
        Collider::circle(30.),
        TransformExtrapolation,
        Transform::from_xyz(-100., -100., 1.),
    ));

    let n = 20;
    let spacing = 50.;
    let offset = spacing * n as f32 / 2.;
    let custom_size = Some(Vec2::new(spacing, spacing));
    for x in 0..n {
        for y in 0..n {
            let x = x as f32 * spacing - offset;
            let y = y as f32 * spacing - offset;
            let color = Color::hsl(240., random::<f32>() * 0.3, random::<f32>() * 0.3);
            commands.spawn((
                Sprite {
                    color,
                    custom_size,
                    ..default()
                },
                Transform::from_xyz(x, y, 0.),
            ));
        }
    }
}

fn binding(trigger: Trigger<Binding<InGame>>, mut players: Query<&mut Actions<InGame>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();

    actions
        .bind::<Move>()
        .to(Cardinal::wasd_keys())
        .with_modifiers(DeadZone::default());

    actions.bind::<Zoom>().to(Input::mouse_wheel());

    actions
        .bind::<PrimaryAttack>()
        .to(Input::MouseButton {
            button: MouseButton::Left,
            mod_keys: ModKeys::empty(),
        })
        .with_conditions(Press::default());

    actions
        .bind::<SecondaryAttack>()
        .to(Input::MouseButton {
            button: MouseButton::Right,
            mod_keys: ModKeys::empty(),
        })
        .with_conditions(Press::default());
}

fn apply_velocity(
    trigger: Trigger<Fired<Move>>,
    mut player: Single<(&mut LinearVelocity, &Player)>,
) {
    player.0.0 = trigger.value * player.1.speed;
}

fn stop_velocity(
    trigger: Trigger<Completed<Move>>,
    mut player: Single<&mut LinearVelocity, With<Player>>,
) {
    player.0 = trigger.value;
}

fn zoom(trigger: Trigger<Fired<Zoom>>, proj: Single<&mut Projection>, mut egui_ctx: EguiContexts) {
    if !egui_ctx.ctx_mut().wants_pointer_input()
        && let Projection::Orthographic(proj) = proj.into_inner().into_inner()
    {
        proj.scale -= trigger.value.y.signum() * 0.1
    }
}

fn primary_attack(
    trigger: Trigger<Fired<PrimaryAttack>>,
    cursor_pos: Res<CursorLocation>,
    transform_q: Query<&Transform>,
    mut commands: Commands,
) {
    let Some(cursor_pos) = cursor_pos.world_position() else {
        return;
    };
    let player_transform = transform_q.get(trigger.target()).unwrap();
    let player_pos = player_transform.translation.xy();
    let direction_vector = cursor_pos - player_pos;

    let normalized_direction_vector = direction_vector.normalize_or_zero();

    let new_point = normalized_direction_vector * 100.;
    let mut new_transform = Transform::from_translation(new_point.extend(0.));
    new_transform.rotation =
        Quat::from_rotation_arc(Vec3::Y, normalized_direction_vector.extend(0.));

    commands.entity(trigger.target()).with_child((
        Collider::rectangle(5., 50.),
        Sensor,
        new_transform,
        CollisionEventsEnabled,
        HitBoxTimer(Timer::from_seconds(0.1, TimerMode::Once)),
    ));
}

fn secondary_attack(
    _: Trigger<Fired<SecondaryAttack>>,
    mark_q: Query<(Entity, &mut Health), (With<Mark>, With<Enemy>)>,
    mut commands: Commands,
) {
    for (entity, mut health) in mark_q {
        health.0 -= 10;
        commands.entity(entity).remove::<Mark>();
    }
}

fn update_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera
        .translation
        .smooth_nudge(&direction, 2., time.delta_secs());
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
            ));
            commands.entity(entity).insert(Moving);
        }
    }
}

fn move_enemies(
    mut commands: Commands,
    enemy_q: Query<
        (Entity, &mut LinearVelocity, &Transform, &Enemy),
        (With<Moving>, Without<Dead>),
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
                    target: new_transform,
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                });
            vel.set_if_neq(LinearVelocity::ZERO);
            continue;
        }

        vel.set_if_neq(LinearVelocity(normalized_direction_vector * enemy.speed));
    }
}

fn check_health(mut commands: Commands, health_q: Query<(Entity, &Health)>) {
    for (entity, health) in health_q {
        if health.0 < 0 {
            commands.entity(entity).despawn();
        }
    }
}