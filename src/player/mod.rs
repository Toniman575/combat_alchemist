mod combat;
mod input;
mod movement;

use avian2d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*};

use bevy_enhanced_input::prelude::*;

use crate::{
    GameCollisionLayer, Health, HealthBar, InGame, Moving,
    player::{
        combat::{
            apply_mark, primary_attack, secondary_attack, trigger_mark, triggers_mark_collision,
        },
        input::{add_mouseover, binding, remove_mouseover},
        movement::{apply_velocity, stop_velocity},
    },
};

pub(super) use crate::player::combat::AppliesMark;
pub(super) use crate::player::combat::AttackMarker;
pub(super) use crate::player::combat::TriggersMark;

#[cfg(debug_assertions)]
use crate::player::{combat::Mark, input::Mouseover};

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_velocity)
            .add_observer(stop_velocity)
            .add_observer(binding)
            .add_observer(primary_attack)
            .add_observer(secondary_attack)
            .add_observer(apply_mark)
            .add_observer(add_mouseover)
            .add_observer(remove_mouseover)
            .add_observer(trigger_mark)
            .add_observer(triggers_mark_collision)
            .add_systems(Startup, startup);

        #[cfg(debug_assertions)]
        app.register_type::<Player>()
            .register_type::<Mark>()
            .register_type::<Mouseover>();
    }
}

#[derive(Component, Reflect)]
#[require(
    Health { current: 100, max: 100 },
    Name::new("Player"),
    RigidBody::Kinematic,
    Collider::circle(25.),
    TransformExtrapolation,
    Actions::<InGame>,
    Transform::from_xyz(0., 0., 1.),
    Moving,
)]
pub struct Player {
    speed: f32,
}

impl Player {
    fn bundle(
        speed: f32,
        mut meshes: ResMut<'_, Assets<Mesh>>,
        mut materials: ResMut<'_, Assets<ColorMaterial>>,
    ) -> impl Bundle {
        (
            Self { speed },
            CollisionLayers::new(
                GameCollisionLayer::Player,
                [GameCollisionLayer::Enemy, GameCollisionLayer::EnemyAttack],
            ),
            children![(
                Mesh2d(meshes.add(Rectangle::new(32., 5.))),
                MeshMaterial2d(materials.add(Color::from(RED))),
                Transform::from_translation(Vec3::new(0., 50., 1.)),
                HealthBar,
                Name::new("Healthbar"),
                Visibility::Visible,
            )],
        )
    }
}

fn startup(
    mut commands: Commands,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
) {
    commands.spawn(Player::bundle(250., meshes, materials));
}
