use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_enoki::Particle2dEffect;
use bevy_seedling::sample::Sample;

#[derive(AssetCollection, Resource)]
pub(super) struct SpriteAssets {
    #[asset(path = "sprites/background.png")]
    pub(super) background: Handle<Image>,
    #[asset(path = "sprites/bite.png")]
    pub(super) bite: Handle<Image>,
    #[asset(path = "sprites/enemy.png")]
    pub(super) enemy: Handle<Image>,
    #[asset(path = "sprites/knob.png")]
    pub(super) knob: Handle<Image>,
    #[asset(path = "sprites/outline.png")]
    pub(super) outline: Handle<Image>,
    #[asset(path = "sprites/player.png")]
    pub(super) player: Handle<Image>,
    #[asset(path = "sprites/potion.png")]
    pub(super) potion: Handle<Image>,
    #[asset(path = "sprites/staff.png")]
    pub(super) staff: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub(super) struct ParticleAssets {
    #[asset(path = "effects/apply_mark.ron")]
    pub(super) apply_mark: Handle<Particle2dEffect>,
    #[asset(path = "effects/mark.ron")]
    pub(super) mark: Handle<Particle2dEffect>,
    #[asset(path = "effects/trigger.ron")]
    pub(super) trigger: Handle<Particle2dEffect>,
}

#[derive(AssetCollection, Resource)]
pub(super) struct AudioAssets {
    #[asset(path = "audio/bite_impact.ogg")]
    pub(super) bite_impact: Handle<Sample>,
    #[asset(path = "audio/bite_swing.ogg")]
    pub(super) bite_swing: Handle<Sample>,
    #[asset(path = "audio/mark_triggered.ogg")]
    pub(super) mark_triggered: Handle<Sample>,
    #[asset(path = "audio/staff_impact.ogg")]
    pub(super) staff_impact: Handle<Sample>,
    #[asset(path = "audio/staff_swing.ogg")]
    pub(super) staff_swing: Handle<Sample>,
}
