use std::time::Duration;

use avian2d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*, sprite::Anchor, time::Stopwatch};

use crate::{
    AssetState, Attacking, GameCollisionLayer, GameState, Health, HealthBar, Moving, SpriteAssets,
    ZLayer, player::Player,
};

pub(super) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(enemy_attack)
            .add_systems(OnEnter(AssetState::Loaded), startup)
            .add_systems(
                Update,
                (move_enemies, spawn_enemies, move_followers).run_if(in_state(GameState::Running)),
            )
            .insert_resource(SpawnTimer(Timer::from_seconds(10., TimerMode::Repeating)));

        #[cfg(debug_assertions)]
        app.register_type::<Enemy>();
    }
}

#[derive(Resource, Reflect, DerefMut, Deref)]
struct SpawnTimer(Timer);

#[derive(Component, Reflect)]
#[require(
    Moving,
    RigidBody::Kinematic,
    Collider::circle(30.),
    TransformExtrapolation,
    CollisionLayers::new(GameCollisionLayer::Enemy, GameCollisionLayer::Player)
)]
pub struct Enemy {
    speed: f32,
}

impl Enemy {
    fn bundle(
        speed: f32,
        health: i16,
        collider_size: f32,
        name: String,
        pos: Vec2,
        sprite_assets: Res<SpriteAssets>,
        mut meshes: ResMut<'_, Assets<Mesh>>,
        mut materials: ResMut<'_, Assets<ColorMaterial>>,
    ) -> impl Bundle {
        (
            Self { speed },
            Health {
                current: health,
                max: health,
            },
            Sprite {
                image: sprite_assets.enemy.clone(),
                anchor: Anchor::Custom(Vec2::new(0., -0.1)),
                custom_size: Some(Vec2::new(20., 20.)),
                ..default()
            },
            CollisionLayers::new(
                GameCollisionLayer::Enemy,
                [[
                    GameCollisionLayer::Enemy,
                    GameCollisionLayer::Player,
                    GameCollisionLayer::PlayerAttack,
                ]],
            ),
            Collider::circle(collider_size),
            Name::new(name),
            Transform::from_translation(pos.extend(ZLayer::Enemies.z_layer())),
            children![(
                Mesh2d(meshes.add(Rectangle::new(25., 2.5))),
                MeshMaterial2d(materials.add(Color::from(RED))),
                Transform::from_translation(Vec3::new(0., 17.5, ZLayer::HealthBar.z_layer())),
                HealthBar,
                Name::new("Healthbar"),
                Visibility::Hidden,
            )],
        )
    }
}

#[derive(Component, Debug)]
#[relationship(relationship_target = FollowedBy)]
pub(super) struct Following(Entity);

impl Following {
    pub(super) fn following(&self) -> Entity {
        self.0
    }
    pub(super) fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

#[derive(Component, Debug)]
#[relationship_target(relationship = Following)]
pub(super) struct FollowedBy(Vec<Entity>);

fn startup(
    mut commands: Commands,
    sprite_assets: Res<SpriteAssets>,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
) {
    commands.spawn(Enemy::bundle(
        30.,
        30,
        8.,
        String::from("Training Dummy"),
        Vec2::new(-100., -100.),
        sprite_assets,
        meshes,
        materials,
    ));
}

fn enemy_attack(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    mut player: Single<(Entity, &mut Health), With<Player>>,
) {
    if player.0 == (trigger.collider) {
        player.1.current -= 10;
        commands.entity(trigger.target()).insert(ColliderDisabled);
    }
}

fn move_enemies(
    mut commands: Commands,
    enemy_q: Query<
        (Entity, &mut LinearVelocity, &Transform, &Enemy),
        (With<Moving>, Without<Attacking>),
    >,
    player: Single<&Transform, With<Player>>,
) {
    for (enemy_entity, mut vel, enemy_transform, enemy) in enemy_q {
        let normalized_direction_vector =
            (player.translation.xy() - enemy_transform.translation.xy()).normalize_or_zero();

        if enemy_transform.translation.distance(player.translation) < 30. {
            let mut new_transform =
                Transform::from_translation((normalized_direction_vector * 80.).extend(0.));
            new_transform.rotation =
                Quat::from_rotation_arc(Vec3::Y, normalized_direction_vector.extend(0.));

            commands
                .entity(enemy_entity)
                .remove::<Moving>()
                .insert(Attacking {
                    hitbox_movement: None,
                    target: normalized_direction_vector,
                    rooted: Duration::from_secs_f32(0.5),
                    spawn_hitbox: vec![Duration::from_secs_f32(0.4)],
                    stopwatch: Stopwatch::new(),
                    range: 25.,
                    hitbox: vec![Collider::rectangle(15., 15.)],
                    hitbox_duration: Duration::from_secs_f32(0.1),
                    movement: None,
                    marker: None,
                });
            vel.set_if_neq(LinearVelocity::ZERO);

            continue;
        }

        vel.set_if_neq(LinearVelocity(normalized_direction_vector * enemy.speed));
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut timer: ResMut<SpawnTimer>,
    time: Res<Time<Virtual>>,
    sprite_assets: Res<SpriteAssets>,
    meshes: ResMut<'_, Assets<Mesh>>,
    materials: ResMut<'_, Assets<ColorMaterial>>,
) {
    if timer.tick(time.delta()).finished() {
        commands.spawn(Enemy::bundle(
            30.,
            30,
            8.,
            String::from("Training Dummy"),
            Vec2::new(-100., -100.),
            sprite_assets,
            meshes,
            materials,
        ));
    }
}

fn move_followers(
    followed_by_q: Query<(&FollowedBy, &Transform), Without<Following>>,
    mut following_q: Query<&mut Transform, With<Following>>,
) {
    for (followed_by, transform) in followed_by_q {
        for entity in followed_by.iter() {
            if let Ok(mut following_transform) = following_q.get_mut(entity) {
                following_transform.set_if_neq(*transform);
            }
        }
    }
}
