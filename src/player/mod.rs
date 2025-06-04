use avian2d::prelude::*;
use bevy::prelude::*;

use bevy_enhanced_input::prelude::*;

mod combat;
mod input;
mod movement;

use crate::{
    GameLayer, Health, InGame,
    player::{
        combat::{apply_mark, primary_attack, secondary_attack},
        input::binding,
        movement::{apply_velocity, stop_velocity},
    },
};

#[cfg(debug_assertions)]
use crate::player::combat::Mark;

pub(super) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_velocity)
            .add_observer(stop_velocity)
            .add_observer(binding)
            .add_observer(primary_attack)
            .add_observer(secondary_attack)
            .add_observer(apply_mark)
            .add_systems(Startup, startup);

        #[cfg(debug_assertions)]
        app.register_type::<Player>().register_type::<Mark>();
    }
}

#[derive(Component, Reflect)]
#[require(
    Health(100),
    Name::new("Player"),
    RigidBody::Kinematic,
    Collider::circle(25.),
    TransformExtrapolation,
    Actions::<InGame>,
    Transform::from_xyz(0., 0., 1.),
)]
pub struct Player {
    speed: f32,
}

impl Player {
    fn bundle(speed: f32) -> impl Bundle {
        (
            Self { speed },
            CollisionLayers::new(
                GameLayer::Player,
                [GameLayer::Enemy, GameLayer::EnemyAttack],
            ),
        )
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(Player::bundle(250.));
}
