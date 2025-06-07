mod combat;
mod input;
mod movement;

use avian2d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*, sprite::Anchor};

use bevy_enhanced_input::prelude::*;

use crate::{
    AssetState, GameCollisionLayer, GameState, Health, HealthBar, InGame, Moving, SpriteAssets,
    ZLayer,
    player::{
        combat::{
            animate_swing, apply_mark, primary_attack, secondary_attack, trigger_mark,
            triggers_mark_collision,
        },
        input::binding,
        movement::{apply_velocity, stop_velocity, weapon_follow},
    },
};

pub(super) use crate::player::combat::AppliesMark;
pub(super) use crate::player::combat::AttackMarker;
pub(super) use crate::player::combat::TriggersMark;

#[cfg(debug_assertions)]
use crate::player::combat::{Mark, Swinging};

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_velocity)
            .add_observer(stop_velocity)
            .add_observer(binding)
            .add_observer(primary_attack)
            .add_observer(secondary_attack)
            .add_observer(apply_mark)
            .add_observer(trigger_mark)
            .add_observer(triggers_mark_collision)
            .add_systems(OnEnter(AssetState::Loaded), startup)
            .add_systems(
                Update,
                (weapon_follow, animate_swing).run_if(in_state(GameState::Running)),
            );

        #[cfg(debug_assertions)]
        app.register_type::<Player>()
            .register_type::<Mark>()
            .register_type::<Swinging>();
    }
}

#[derive(Component, Reflect)]
#[require(
    Health { current: 100, max: 100 },
    Name::new("Player"),
    RigidBody::Kinematic,
    Collider::circle(5.),
    TransformExtrapolation,
    Actions::<InGame>,
    Transform::from_xyz(0., 0., ZLayer::Player.z_layer()),
    Moving,
)]
pub struct Player {
    speed: f32,
}

impl Player {
    fn bundle(
        speed: f32,
        sprite_assets: Res<SpriteAssets>,
        mut meshes: ResMut<'_, Assets<Mesh>>,
        mut materials: ResMut<'_, Assets<ColorMaterial>>,
    ) -> impl Bundle {
        (
            Self { speed },
            Sprite {
                image: sprite_assets.player.clone(),
                anchor: Anchor::Custom(Vec2::new(0., -0.2)),
                ..default()
            },
            CollisionLayers::new(
                GameCollisionLayer::Player,
                [GameCollisionLayer::Enemy, GameCollisionLayer::EnemyAttack],
            ),
            children![(
                Mesh2d(meshes.add(Rectangle::new(15., 2.5))),
                MeshMaterial2d(materials.add(Color::from(RED))),
                Transform::from_translation(Vec3::new(0., 17.5, ZLayer::HealthBar.z_layer())),
                HealthBar,
                Name::new("Healthbar"),
                Visibility::Visible,
            )],
        )
    }
}

#[derive(Component, Reflect)]
struct WeaponSprite;

fn startup(
    mut commands: Commands,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
    sprite_assets: Res<SpriteAssets>,
) {
    commands.spawn((
        Sprite {
            image: sprite_assets.staff.clone(),
            ..default()
        },
        Transform::from_xyz(0., 0., ZLayer::PlayerWeapon.z_layer()),
        WeaponSprite,
    ));
    commands.spawn(Player::bundle(50., sprite_assets, meshes, materials));
}
