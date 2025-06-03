use avian2d::prelude::*;
use bevy::prelude::*;

use bevy_enhanced_input::prelude::*;

use crate::player::{Player, input::MovePlayer};

pub(super) fn apply_velocity(
    trigger: Trigger<Fired<MovePlayer>>,
    mut player: Single<(&mut LinearVelocity, &Player)>,
) {
    player.0.0 = trigger.value * player.1.speed;
}

pub(super) fn stop_velocity(
    trigger: Trigger<Completed<MovePlayer>>,
    mut player: Single<&mut LinearVelocity, With<Player>>,
) {
    player.0 = trigger.value;
}
