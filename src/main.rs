mod assets;
mod audio;
mod camera;
mod combat;
mod enemy;
mod movement;
mod player;
mod touch;

use avian2d::prelude::*;
use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResolution};
use bevy_asset_loader::prelude::*;
use bevy_cursor::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_enoki::EnokiPlugin;
#[cfg(debug_assertions)]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_seedling::SeedlingPlugin;
use virtual_joystick::VirtualJoystickPlugin;

use crate::{
    assets::{AudioAssets, ParticleAssets, SpriteAssets},
    audio::spawn_collision_sound,
    camera::CameraPlugin,
    combat::{
        AttackMovements, Health, HealthBar, animate_swing, attacking_movement, tick_attack_timer,
        tick_hitbox_timer,
    },
    enemy::{Enemy, EnemyPlugin},
    movement::{Rooted, kinematic_collisions, tick_rooted},
    player::{AppliesMark, AttackMarker, JoystickID, PlayerPlugin, TriggersMark},
    touch::touch_interface,
};

#[cfg(debug_assertions)]
use crate::{
    audio::HitboxSound,
    combat::{AttackHitBoxTimer, Swings},
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

#[derive(InputContext)]
struct InGame;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Pause;

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
            animate_swing,
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
    .register_type::<HitboxSound>()
    .register_type::<Swings>();

    app.run()
}

fn startup(mut commands: Commands, sprite_asset: Res<SpriteAssets>) {
    commands.spawn((
        Name::new("Map"),
        Sprite {
            image: sprite_asset.background.clone_weak(),
            anchor: bevy::sprite::Anchor::Center,
            custom_size: Some(Vec2::new(1600., 1600.)),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.,
            },
            ..default()
        },
        Transform::from_xyz(0., 0., ZLayer::Map.z_layer()),
    ));
}

fn binding(trigger: Trigger<Binding<InGame>>, mut players: Query<&mut Actions<InGame>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Pause>()
        .to(Input::Keyboard {
            key: KeyCode::Escape,
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
