use avian2d::prelude::OnCollisionStart;
use bevy::prelude::*;
use bevy_seedling::sample::{Sample, SamplePlayer};

#[derive(Component, Reflect, DerefMut, Deref)]
pub(super) struct HitboxSound(pub(super) Handle<Sample>);

pub(super) fn spawn_collision_sound(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    sound_q: Query<&HitboxSound>,
) {
    if let Ok(sound) = sound_q.get(trigger.target()) {
        commands.spawn(SamplePlayer::new(sound.0.clone_weak()));
    }
}
