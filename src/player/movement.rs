use avian2d::prelude::*;
use bevy::prelude::*;

use bevy_enhanced_input::prelude::*;

use crate::{
    Moving, ZLayer,
    player::{Player, WeaponSprite, combat::Swinging, input::MovePlayer},
};

#[derive(Component, Reflect, Default, Deref, DerefMut)]
pub(super) struct LookingDirection(pub(super) Vec2);

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
    player: Single<(&Transform, &LookingDirection), With<Player>>,
    mut weapon: Single<&mut Transform, (With<WeaponSprite>, (Without<Player>, Without<Swinging>))>,
    time: Res<Time<Virtual>>,
) {
    let target =
        &(player.0.translation.xy() + Vec2::new(5.0, 5.0)).extend(ZLayer::PlayerWeapon.z_layer());
    weapon
        .translation
        .smooth_nudge(target, 10., time.delta_secs());

    let target_rotation = Quat::from_rotation_arc(
        Vec3::Y,
        ((player.1.0 * 100. + player.0.translation.truncate()) - target.truncate())
            .normalize()
            .extend(0.),
    );
    weapon
        .rotation
        .smooth_nudge(&target_rotation, 10., time.delta_secs());
}
