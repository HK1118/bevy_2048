use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};

use super::animation::AnimationPhase;

#[derive(Resource, Clone)]
pub(super) struct IdleFocusedUpdateMode(UpdateMode);

pub(super) fn capture_idle_update_mode(mut commands: Commands, winit_settings: Res<WinitSettings>) {
    commands.insert_resource(IdleFocusedUpdateMode(winit_settings.focused_mode));
}

pub(super) fn request_redraw_during_animation(
    phase: Res<AnimationPhase>,
    mut redraw: MessageWriter<bevy::window::RequestRedraw>,
) {
    if *phase != AnimationPhase::Idle {
        redraw.write(bevy::window::RequestRedraw);
    }
}

pub(super) fn sync_focused_update_mode(
    phase: Res<AnimationPhase>,
    idle_mode: Res<IdleFocusedUpdateMode>,
    mut winit_settings: ResMut<WinitSettings>,
) {
    let desired_mode = if *phase == AnimationPhase::Idle {
        idle_mode.0
    } else {
        UpdateMode::Continuous
    };

    if winit_settings.focused_mode != desired_mode {
        winit_settings.focused_mode = desired_mode;
    }
}
