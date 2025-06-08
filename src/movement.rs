use std::time::Duration;

use avian2d::{
    math::{AdjustPrecision, Scalar},
    prelude::{ColliderOf, Collisions, LinearVelocity, Position, RigidBody, Sensor},
};
use bevy::{prelude::*, time::Stopwatch};

#[derive(Component, Reflect, Default)]
pub struct Moving;

#[derive(Component, Reflect)]
pub(super) struct Rooted {
    pub(super) duration: Duration,
    pub(super) stopwatch: Stopwatch,
}

pub(super) fn tick_rooted(
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
#[allow(clippy::type_complexity)]
pub(super) fn kinematic_collisions(
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
