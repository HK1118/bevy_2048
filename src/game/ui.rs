use bevy::prelude::*;
use rand::rng;

use super::animation::{AnimationPhase, PendingSlide};
use super::board::{Board, Score};
use super::{GamePhase, HasWon};
use super::render::{spawn_visual_tile, VisualTile};
use super::GameFont;

#[derive(Component)]
pub(super) struct UIScoreText;

#[derive(Component)]
pub(super) struct NewGameButton;

#[derive(Component)]
pub(super) struct OverlayRoot;

const BUTTON_BG: Color = Color::srgb(0.557, 0.494, 0.439);
const BUTTON_HOVER: Color = Color::srgb(0.647, 0.584, 0.529);
const SCORE_COLOR: Color = Color::srgb(0.467, 0.431, 0.396);
const OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);

pub(super) fn setup_ui(mut commands: Commands, font: Res<GameFont>) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            ..default()
        })
        .with_children(|parent| {
            // スコアテキスト
            parent.spawn((
                UIScoreText,
                Text::new("Score: 0"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(SCORE_COLOR),
            ));

            // New Game ボタン
            parent
                .spawn((
                    NewGameButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(BUTTON_BG),
                    children![(
                        Text::new("New Game"),
                        TextFont {
                            font: font.0.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    )],
                ))
                .observe(on_new_game_click);
        });
}

fn on_new_game_click(
    _click: On<Pointer<Click>>,
    mut commands: Commands,
    mut board: ResMut<Board>,
    mut score: ResMut<Score>,
    mut phase: ResMut<AnimationPhase>,
    mut pending: ResMut<PendingSlide>,
    mut has_won: ResMut<HasWon>,
    mut next_state: ResMut<NextState<GamePhase>>,
    font: Res<GameFont>,
    tiles: Query<Entity, With<VisualTile>>,
) {
    for entity in &tiles {
        commands.entity(entity).despawn();
    }

    *board = Board::with_two_tiles(&mut rng());
    **score = 0;
    *phase = AnimationPhase::Idle;
    *pending = PendingSlide::default();
    has_won.0 = false;
    next_state.set(GamePhase::Playing);

    for (index, cell) in board.iter().enumerate() {
        if let Some(exp) = cell {
            spawn_visual_tile(&mut commands, index, *exp, Vec3::ONE, &font);
        }
    }
}

fn on_continue_click(
    _click: On<Pointer<Click>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    next_state.set(GamePhase::Playing);
}

fn spawn_overlay_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    label: &str,
    font: &Handle<Font>,
) -> EntityCommands<'a> {
    parent.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(BUTTON_BG),
        children![(
            Text::new(label.to_string()),
            TextFont {
                font: font.clone(),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
        )],
    ))
}

fn spawn_overlay(
    commands: &mut Commands,
    title: &str,
    score_value: u32,
    show_continue: bool,
    font: &Handle<Font>,
) {
    commands
        .spawn((
            OverlayRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(OVERLAY_BG),
            // Z-index で他のUIの上に表示
            ZIndex(10),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(32.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|parent| {
                    // タイトル
                    parent.spawn((
                        Text::new(title),
                        TextFont {
                            font: font.clone(),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // スコア
                    parent.spawn((
                        Text::new(format!("Score: {score_value}")),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                    ));

                    // ボタン行
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        })
                        .with_children(|parent| {
                            if show_continue {
                                spawn_overlay_button(parent, "Continue", font)
                                    .observe(on_continue_click);
                            }
                            spawn_overlay_button(parent, "New Game", font)
                                .observe(on_new_game_click);
                        });
                });
        });
}

pub(super) fn spawn_game_over_overlay(
    mut commands: Commands,
    score: Res<Score>,
    font: Res<GameFont>,
) {
    spawn_overlay(&mut commands, "Game Over", **score, false, &font.0);
}

pub(super) fn spawn_won_overlay(
    mut commands: Commands,
    score: Res<Score>,
    font: Res<GameFont>,
) {
    spawn_overlay(&mut commands, "You Win!", **score, true, &font.0);
}

pub(super) fn despawn_overlay(
    mut commands: Commands,
    overlay: Query<Entity, With<OverlayRoot>>,
) {
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
}

pub(super) fn button_hover(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in &mut query {
        *color = match *interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(BUTTON_HOVER),
            Interaction::None => BackgroundColor(BUTTON_BG),
        };
    }
}

pub(super) fn sync_ui_score(score: Res<Score>, mut query: Query<&mut Text, With<UIScoreText>>) {
    if !score.is_changed() {
        return;
    }

    for mut text in &mut query {
        text.0 = format!("Score: {}", **score);
    }
}
