use bevy::prelude::*;

use bevy_enhanced_input::prelude::*;

#[cfg(debug_assertions)]
use bevy_inspector_egui::bevy_egui::EguiContexts;

use crate::{AssetState, GameState, InGame, player::Player};

pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(zoom)
            .add_observer(binding)
            .add_systems(OnEnter(AssetState::Loaded), startup)
            .add_systems(Update, update_camera.run_if(in_state(GameState::Running)));
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Zoom;

fn startup(mut commands: Commands) {
    let mut proj = OrthographicProjection::default_2d();
    proj.scale = 0.2;
    commands.spawn((Camera2d, Projection::Orthographic(proj)));
}

#[cfg(debug_assertions)]
fn zoom(trigger: Trigger<Fired<Zoom>>, proj: Single<&mut Projection>, mut egui_ctx: EguiContexts) {
    if !egui_ctx.ctx_mut().wants_pointer_input()
        && let Projection::Orthographic(proj) = proj.into_inner().into_inner()
    {
        proj.scale = (proj.scale - trigger.value.y.signum() * 0.025).clamp(0.1, 0.6)
    }
}

#[cfg(not(debug_assertions))]
fn zoom(trigger: Trigger<Fired<Zoom>>, proj: Single<&mut Projection>) {
    if let Projection::Orthographic(proj) = proj.into_inner().into_inner() {
        proj.scale -= trigger.value.y.signum() * 0.1
    }
}

fn update_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time<Virtual>>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera
        .translation
        .smooth_nudge(&direction, 2., time.delta_secs());
}

fn binding(trigger: Trigger<Binding<InGame>>, mut players: Query<&mut Actions<InGame>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions.bind::<Zoom>().to(Input::mouse_wheel());
}
