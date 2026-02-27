use bevy::prelude::*;
use rand::rng;

use super::GameFont;
use super::board::{Board, Score, SlideResult};
use super::input::Slide;
use super::render::{VisualTile, board_index_to_position, spawn_visual_tile};

const SLIDE_DURATION: f32 = 0.15;
const EFFECT_DURATION: f32 = 0.1;
const MERGE_SCALE_PEAK: f32 = 1.2;

#[derive(Resource, Default, PartialEq, Debug)]
pub(super) enum AnimationPhase {
    #[default]
    Idle,
    Sliding,
    Settling,
}

#[derive(Resource, Default)]
pub(super) struct PendingSlide(Option<SlideResult>);

#[derive(Component)]
pub(super) struct SlideAnim {
    from: Vec2,
    to: Vec2,
    timer: Timer,
}

#[derive(Component)]
pub(super) struct MergeAnim(Timer);

#[derive(Component)]
pub(super) struct SpawnAnim(Timer);

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Slide メッセージを受け取り、アニメーションを開始する
pub(super) fn prepare_slide(
    mut move_reader: MessageReader<Slide>,
    board: Res<Board>,
    mut phase: ResMut<AnimationPhase>,
    mut pending: ResMut<PendingSlide>,
    tiles: Query<(Entity, &VisualTile)>,
    mut commands: Commands,
) {
    if *phase != AnimationPhase::Idle {
        move_reader.read().for_each(drop);
        return;
    }

    for Slide(direction) in move_reader.read() {
        let result = board.compute_slide(*direction);
        if !result.changed {
            continue;
        }

        for movement in &result.movements {
            for (entity, tile) in &tiles {
                if tile.board_index == movement.from {
                    let from_pos = board_index_to_position(movement.from);
                    let to_pos = board_index_to_position(movement.to);

                    if from_pos != to_pos {
                        commands.entity(entity).insert(SlideAnim {
                            from: from_pos,
                            to: to_pos,
                            timer: Timer::from_seconds(SLIDE_DURATION, TimerMode::Once),
                        });
                    }
                    break;
                }
            }
        }

        pending.0 = Some(result);
        *phase = AnimationPhase::Sliding;
        break;
    }
}

/// スライドアニメーションを進行する
pub(super) fn animate_slide(
    time: Res<Time>,
    phase: Res<AnimationPhase>,
    mut tiles: Query<(&mut Transform, &mut SlideAnim)>,
) {
    if *phase != AnimationPhase::Sliding {
        return;
    }

    for (mut transform, mut anim) in &mut tiles {
        anim.timer.tick(time.delta());
        let t = ease_out_cubic(anim.timer.fraction());
        let pos = anim.from.lerp(anim.to, t);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

/// スライド完了後に Board を更新し、マージ/出現エフェクトを開始する
pub(super) fn resolve_slide(
    mut commands: Commands,
    mut phase: ResMut<AnimationPhase>,
    mut board: ResMut<Board>,
    mut score: ResMut<Score>,
    mut pending: ResMut<PendingSlide>,
    font: Res<GameFont>,
    tiles_with_anim: Query<&SlideAnim, With<VisualTile>>,
    all_tiles: Query<Entity, With<VisualTile>>,
) {
    if *phase != AnimationPhase::Sliding {
        return;
    }

    if tiles_with_anim.iter().any(|anim| !anim.timer.is_finished()) {
        return;
    }

    let Some(result) = pending.0.take() else {
        return;
    };

    let merge_dests = result.merge_destinations;

    // Board 更新
    *board = result.new_board;
    **score += result.score_gained;

    // 既存タイルをすべて削除
    for entity in &all_tiles {
        commands.entity(entity).despawn();
    }

    // 新しいタイルをスポーン（マージ先にはパルスアニメーション）
    for (index, cell) in board.iter().enumerate() {
        if let Some(exp) = cell {
            let entity = spawn_visual_tile(&mut commands, index, *exp, Vec3::ONE, &font);
            if merge_dests.contains(&index) {
                commands
                    .entity(entity)
                    .insert(MergeAnim(Timer::from_seconds(
                        EFFECT_DURATION,
                        TimerMode::Once,
                    )));
            }
        }
    }

    // ランダムタイルを配置（出現アニメーション付き）
    if let Some(idx) = board.place_random_tile(&mut rng())
        && let Some(exp) = board[idx]
    {
        let entity = spawn_visual_tile(&mut commands, idx, exp, Vec3::ZERO, &font);
        commands
            .entity(entity)
            .insert(SpawnAnim(Timer::from_seconds(
                EFFECT_DURATION,
                TimerMode::Once,
            )));
    }

    *phase = AnimationPhase::Settling;
}

/// マージパルスと出現アニメーションを進行する
pub(super) fn animate_effects(
    time: Res<Time>,
    mut phase: ResMut<AnimationPhase>,
    mut merge_tiles: Query<(&mut Transform, &mut MergeAnim), Without<SpawnAnim>>,
    mut spawn_tiles: Query<(&mut Transform, &mut SpawnAnim), Without<MergeAnim>>,
) {
    if *phase != AnimationPhase::Settling {
        return;
    }

    let mut all_done = true;

    for (mut transform, mut anim) in &mut merge_tiles {
        anim.0.tick(time.delta());
        let t = anim.0.fraction();
        let scale = 1.0 + (MERGE_SCALE_PEAK - 1.0) * (t * std::f32::consts::PI).sin();
        transform.scale = Vec3::splat(scale);
        if !anim.0.is_finished() {
            all_done = false;
        }
    }

    for (mut transform, mut anim) in &mut spawn_tiles {
        anim.0.tick(time.delta());
        let t = ease_out_cubic(anim.0.fraction());
        transform.scale = Vec3::splat(t);
        if !anim.0.is_finished() {
            all_done = false;
        }
    }

    if all_done {
        *phase = AnimationPhase::Idle;
    }
}
