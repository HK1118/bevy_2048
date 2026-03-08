mod animation;
mod board;
mod input;
mod render;
mod ui;

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use rand::rng;

use animation::{AnimationPhase, PendingSlide};
use board::{Board, Score};
use input::{Slide, handle_input, on_drag_end};

#[derive(Resource)]
pub(super) struct GameFont(pub(super) Handle<Font>);

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub(super) enum GamePhase {
    #[default]
    Playing,
    Won,
    GameOver,
}

#[derive(Resource, Default)]
pub(super) struct HasWon(pub(super) bool);

#[derive(Resource, Clone)]
struct IdleFocusedUpdateMode(UpdateMode);

pub(super) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .register_type::<Score>()
            .insert_resource(Board::with_two_tiles(&mut rng()))
            .register_type::<Board>()
            .init_resource::<AnimationPhase>()
            .init_resource::<PendingSlide>()
            .init_resource::<HasWon>()
            .init_state::<GamePhase>()
            .add_message::<Slide>()
            .add_observer(on_drag_end)
            .add_systems(
                Startup,
                (
                    load_font,
                    capture_idle_update_mode,
                    render::setup_board,
                    ui::setup_ui,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    handle_input,
                    animation::prepare_slide,
                    animation::animate_slide,
                    animation::resolve_slide,
                    animation::animate_effects,
                    check_game_state,
                    request_redraw_during_animation,
                    sync_focused_update_mode,
                )
                    .chain()
                    .run_if(in_state(GamePhase::Playing)),
            )
            .add_systems(
                Update,
                (
                    ui::sync_ui_score,
                    ui::button_hover,
                    ui::adapt_header_to_window,
                ),
            )
            .add_systems(OnEnter(GamePhase::GameOver), ui::spawn_game_over_overlay)
            .add_systems(OnEnter(GamePhase::Won), ui::spawn_won_overlay)
            .add_systems(OnExit(GamePhase::GameOver), ui::despawn_overlay)
            .add_systems(OnExit(GamePhase::Won), ui::despawn_overlay);
    }
}

fn load_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/DotGothic16-Regular.ttf");
    commands.insert_resource(GameFont(font));
}

fn capture_idle_update_mode(mut commands: Commands, winit_settings: Res<WinitSettings>) {
    commands.insert_resource(IdleFocusedUpdateMode(winit_settings.focused_mode));
}

fn check_game_state(
    board: Res<Board>,
    phase: Res<AnimationPhase>,
    mut has_won: ResMut<HasWon>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    if *phase != AnimationPhase::Idle {
        return;
    }

    // 2048 到達チェック（初回のみ）
    if !has_won.0 && board.iter().any(|cell| cell.map(|e| e.get()) == Some(11)) {
        has_won.0 = true;
        next_state.set(GamePhase::Won);
        return;
    }

    // ゲームオーバーチェック
    if !board.can_move() {
        next_state.set(GamePhase::GameOver);
    }
}

fn request_redraw_during_animation(
    phase: Res<AnimationPhase>,
    mut redraw: MessageWriter<bevy::window::RequestRedraw>,
) {
    if *phase != AnimationPhase::Idle {
        redraw.write(bevy::window::RequestRedraw);
    }
}

fn sync_focused_update_mode(
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
