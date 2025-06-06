use avian2d::prelude::*;
use bevy::prelude::*;

use bevy_cursor::CursorLocation;
use bevy_enhanced_input::prelude::*;

use crate::{
    Moving, ZLayer,
    player::{Player, WeaponSprite, combat::Swinging, input::MovePlayer},
};

pub(super) fn apply_velocity(
    trigger: Trigger<Fired<MovePlayer>>,
    mut player: Single<(&mut LinearVelocity, &Player), With<Moving>>,
) {
    player.0.0 = trigger.value * player.1.speed;
}

pub(super) fn stop_velocity(
    trigger: Trigger<Completed<MovePlayer>>,
    mut player: Single<&mut LinearVelocity, (With<Player>, With<Moving>)>,
) {
    player.0 = trigger.value;
}

pub(super) fn weapon_follow(
    cursor: Res<CursorLocation>,
    player: Single<&Transform, With<Player>>,
    mut weapon: Single<&mut Transform, (With<WeaponSprite>, (Without<Player>, Without<Swinging>))>,
    time: Res<Time<Virtual>>,
) {
    let target =
        &(player.translation.xy() + Vec2::new(5.0, 5.0)).extend(ZLayer::PlayerWeapon.z_layer());

    weapon
        .translation
        .smooth_nudge(target, 10., time.delta_secs());

    let Some(cursor_pos) = cursor.world_position() else {
        return;
    };

    let target_rotation = Quat::from_rotation_arc(
        Vec3::Y,
        (cursor_pos - target.truncate()).normalize().extend(0.),
    );
    weapon
        .rotation
        .smooth_nudge(&target_rotation, 10., time.delta_secs());
}
