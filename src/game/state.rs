use bevy::prelude::*;

use super::animation::AnimationPhase;
use super::board::Board;

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub(super) enum GamePhase {
    #[default]
    Playing,
    Won,
    GameOver,
}

#[derive(Resource, Default)]
pub(super) struct HasWon(pub(super) bool);

pub(super) fn check_game_state(
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
