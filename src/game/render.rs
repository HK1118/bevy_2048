use std::num::NonZero;

use bevy::camera::ScalingMode;
use bevy::prelude::*;

use super::GameFont;
use super::board::{BOARD_SIZE, Board, exp_to_value};

pub(super) const TILE_SIZE: f32 = 100.0;
pub(super) const TILE_GAP: f32 = 10.0;
const BOARD_PADDING: f32 = 10.0;
pub(super) const BOARD_PX: f32 =
    TILE_SIZE * BOARD_SIZE as f32 + TILE_GAP * (BOARD_SIZE as f32 + 1.0);
pub(super) const BOARD_OFFSET_Y: f32 = -30.0;
const MARGIN: f32 = 40.0;
const HEADER_HEIGHT: f32 = 80.0;

/// Text2d を高解像度でラスタライズするためのスケール倍率。
/// font_size にこの値を掛け、Transform を 1/この値 に縮小することで、
/// カメラ拡大時でもテキストがクリアに表示される。
const TEXT_RENDER_SCALE: f32 = 3.0;

const COLOR_BG: Color = Color::srgb(0.98, 0.97, 0.94);
const COLOR_BOARD: Color = Color::srgb(0.733, 0.678, 0.627);
const COLOR_EMPTY_CELL: Color = Color::srgb(0.804, 0.757, 0.706);
const COLOR_TEXT_DARK: Color = Color::srgb(0.467, 0.431, 0.396);

#[derive(Component)]
struct CellBackground;

#[derive(Component)]
pub(super) struct VisualTile {
    pub(super) board_index: usize,
}

#[derive(Component)]
pub(super) struct TileText;

/// ボードインデックスからワールド座標を計算する
pub(super) fn board_index_to_position(index: usize) -> Vec2 {
    let x = index % BOARD_SIZE;
    let y = index / BOARD_SIZE;
    let offset = -BOARD_PX / 2.0 + BOARD_PADDING + TILE_SIZE / 2.0;
    Vec2::new(
        offset + x as f32 * (TILE_SIZE + TILE_GAP),
        offset + y as f32 * (TILE_SIZE + TILE_GAP) + BOARD_OFFSET_Y,
    )
}

pub(super) fn tile_color(exp: Option<NonZero<u8>>) -> Color {
    match exp.map(|e| e.get()) {
        None => COLOR_EMPTY_CELL,
        Some(1) => Color::srgb(0.933, 0.894, 0.855), // 2
        Some(2) => Color::srgb(0.933, 0.882, 0.788), // 4
        Some(3) => Color::srgb(0.953, 0.698, 0.478), // 8
        Some(4) => Color::srgb(0.965, 0.588, 0.392), // 16
        Some(5) => Color::srgb(0.969, 0.486, 0.373), // 32
        Some(6) => Color::srgb(0.969, 0.373, 0.231), // 64
        Some(7) => Color::srgb(0.929, 0.816, 0.451), // 128
        Some(8) => Color::srgb(0.929, 0.800, 0.384), // 256
        Some(9) => Color::srgb(0.929, 0.788, 0.314), // 512
        Some(10) => Color::srgb(0.929, 0.773, 0.247), // 1024
        Some(11) => Color::srgb(0.929, 0.761, 0.180), // 2048
        _ => Color::srgb(0.239, 0.227, 0.196),       // > 2048
    }
}

pub(super) fn text_color(exp: Option<NonZero<u8>>) -> Color {
    match exp.map(|e| e.get()) {
        Some(1 | 2) => COLOR_TEXT_DARK,
        _ => Color::srgb(0.976, 0.965, 0.949),
    }
}

pub(super) fn font_size_for_tile(exp: Option<NonZero<u8>>) -> f32 {
    match exp.map(|e| e.get()) {
        Some(e) if e >= 10 => 26.0, // 1024+
        Some(e) if e >= 7 => 32.0,  // 128–512
        Some(e) if e >= 4 => 36.0,  // 16–64
        _ => 40.0,                  // 2, 4, 8
    }
}

/// VisualTile エンティティをスポーンする
pub(super) fn spawn_visual_tile(
    commands: &mut Commands,
    board_index: usize,
    exp: NonZero<u8>,
    scale: Vec3,
    font: &GameFont,
) -> Entity {
    let pos = board_index_to_position(board_index);
    let tile = Some(exp);

    commands
        .spawn((
            VisualTile { board_index },
            Sprite {
                color: tile_color(tile),
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            Transform::from_translation(pos.extend(2.0)).with_scale(scale),
        ))
        .with_children(|parent| {
            let inv_scale = 1.0 / TEXT_RENDER_SCALE;
            parent.spawn((
                TileText,
                Text2d::new(exp_to_value(exp.get()).to_string()),
                TextFont {
                    font: font.0.clone(),
                    font_size: font_size_for_tile(tile) * TEXT_RENDER_SCALE,
                    ..default()
                },
                TextColor(text_color(tile)),
                TextLayout::new_with_justify(Justify::Center),
                Transform::from_translation(Vec3::Z).with_scale(Vec3::splat(inv_scale)),
            ));
        })
        .id()
}

pub(super) fn setup_board(mut commands: Commands, board: Res<Board>, font: Res<GameFont>) {
    commands.insert_resource(ClearColor(COLOR_BG));
    commands.spawn((
        Camera2d,
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: BOARD_PX + MARGIN * 2.0,
                min_height: BOARD_PX + HEADER_HEIGHT + MARGIN * 2.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    // ボード背景
    commands.spawn((
        Sprite {
            color: COLOR_BOARD,
            custom_size: Some(Vec2::splat(BOARD_PX)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, BOARD_OFFSET_Y, 0.0)),
    ));

    // 16 個のセル背景
    for index in 0..(BOARD_SIZE * BOARD_SIZE) {
        let pos = board_index_to_position(index);
        commands.spawn((
            CellBackground,
            Sprite {
                color: COLOR_EMPTY_CELL,
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            Transform::from_translation(pos.extend(1.0)),
        ));
    }

    // 初期タイルのスポーン
    for (index, cell) in board.iter().enumerate() {
        if let Some(exp) = cell {
            spawn_visual_tile(&mut commands, index, *exp, Vec3::ONE, &font);
        }
    }
}
