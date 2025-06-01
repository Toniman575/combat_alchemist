use avian2d::{
    math::{AdjustPrecision, Scalar},
    prelude::*,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(InputContext)]
struct InGame;

#[derive(Component, Reflect)]
struct Player;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
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
        .add_observer(binding)
        .insert_resource(Gravity::ZERO)
        .add_systems(Startup, startup)
        .add_systems(Update, kinematic_controller_collisions)
        .register_type::<Player>()
        .run()
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Name::new("Player"),
        Player,
        RigidBody::Kinematic,
        Collider::circle(25.),
        TransformExtrapolation,
        Actions::<InGame>::default(),
    ));
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(100., 100.),
        Transform::from_xyz(100., 100., 0.),
    ));
}

fn binding(trigger: Trigger<Binding<InGame>>, mut players: Query<&mut Actions<InGame>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();

    actions
        .bind::<Move>()
        .to(Cardinal::wasd_keys())
        .with_modifiers((
            DeadZone::default(),
            SmoothNudge::default(),
            Scale::splat(250.),
        ));
}

fn apply_velocity(
    trigger: Trigger<Fired<Move>>,
    mut player: Single<&mut LinearVelocity, With<Player>>,
) {
    player.0 = trigger.value;
}

fn kinematic_controller_collisions(
    collisions: Collisions,
    bodies: Query<&RigidBody>,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut character_controllers: Query<(&mut Position, &mut LinearVelocity), With<Player>>,
    time: Res<Time>,
) {
    for contacts in collisions.iter() {
        let Ok([&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }]) =
            collider_rbs.get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        let is_first: bool;

        let character_rb: RigidBody;
        let is_other_dynamic: bool;

        let (mut position, mut linear_velocity) =
            if let Ok(character) = character_controllers.get_mut(rb1) {
                is_first = true;
                character_rb = *bodies.get(rb1).unwrap();
                is_other_dynamic = bodies.get(rb2).is_ok_and(|rb| rb.is_dynamic());
                character
            } else if let Ok(character) = character_controllers.get_mut(rb2) {
                is_first = false;
                character_rb = *bodies.get(rb2).unwrap();
                is_other_dynamic = bodies.get(rb1).is_ok_and(|rb| rb.is_dynamic());
                character
            } else {
                continue;
            };

        if !character_rb.is_kinematic() {
            continue;
        }

        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.normal
            } else {
                manifold.normal
            };

            let mut deepest_penetration: Scalar = Scalar::MIN;

            for contact in manifold.points.iter() {
                if contact.penetration > 0.0 {
                    position.0 += normal * contact.penetration;
                }
                deepest_penetration = deepest_penetration.max(contact.penetration);
            }

            if is_other_dynamic {
                continue;
            }

            if deepest_penetration > 0.0 {
                if linear_velocity.dot(normal) > 0.0 {
                    continue;
                }

                let impulse = linear_velocity.reject_from_normalized(normal);
                linear_velocity.0 = impulse;
            } else {
                let normal_speed = linear_velocity.dot(normal);

                if normal_speed > 0.0 {
                    continue;
                }

                let impulse_magnitude =
                    normal_speed - (deepest_penetration / time.delta_secs_f64().adjust_precision());
                let mut impulse = impulse_magnitude * normal;

                impulse.y = impulse.y.max(0.0);
                linear_velocity.0 -= impulse;
            }
        }
    }
}
