use std::{fmt, num::NonZero};

use bevy::prelude::*;
use rand::prelude::*;

pub(super) static BOARD_SIZE: usize = 4;

pub(super) fn non_zero_exp(exp: u8) -> NonZero<u8> {
    NonZero::new(exp).expect("tile exponent must be non-zero")
}

pub(super) fn exp_to_value(exp: u8) -> u32 {
    2u32.pow(u32::from(exp))
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SlideMovement {
    pub(super) from: usize,
    pub(super) to: usize,
}

#[derive(Debug, Clone)]
pub(super) struct SlideResult {
    pub(super) changed: bool,
    pub(super) movements: Vec<SlideMovement>,
    pub(super) merge_destinations: Vec<usize>,
    pub(super) new_board: Board,
    pub(super) score_gained: u32,
}

#[derive(Resource, Default, Clone, Copy, Deref, DerefMut, Reflect, Debug)]
#[reflect(Resource)]
pub(super) struct Score(pub(super) u32);

#[derive(Clone, Copy, Debug)]
pub(super) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// スライド先の端が先頭になるよう、行/列 `i` のインデックス列を返す。
    /// `slide_line` は先頭に向かってタイルを詰めるため、この順序でインデックスを並べる。
    fn line_indices(&self, i: usize) -> [usize; BOARD_SIZE] {
        std::array::from_fn(|j| match self {
            Self::Left => Board::index(j, i),
            Self::Right => Board::index(BOARD_SIZE - 1 - j, i),
            Self::Up => Board::index(i, BOARD_SIZE - 1 - j),
            Self::Down => Board::index(i, j),
        })
    }
}

#[derive(Resource, Default, Clone, Deref, DerefMut, Reflect, Debug)]
#[reflect(Resource)]
pub(super) struct Board([Option<NonZero<u8>>; BOARD_SIZE * BOARD_SIZE]);

impl Board {
    pub(super) fn with_two_tiles<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let mut board = Self::default();
        board.place_random_tile(rng);
        board.place_random_tile(rng);
        board
    }

    pub(super) fn place_random_tile<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<usize> {
        let mut selected = None;
        let mut empty_count = 0usize;
        for (index, cell) in self.iter().enumerate() {
            if cell.is_none() {
                empty_count += 1;
                if rng.random_range(0..empty_count) == 0 {
                    selected = Some(index);
                }
            }
        }

        if let Some(index) = selected {
            let value = if rng.random_range(0..10) == 0 { 2 } else { 1 };
            self[index] = Some(non_zero_exp(value));
        }
        selected
    }

    fn index(x: usize, y: usize) -> usize {
        x + y * BOARD_SIZE
    }

    pub(super) fn can_move(&self) -> bool {
        if self.iter().any(Option::is_none) {
            return true;
        }

        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let i = Self::index(x, y);
                let current = self[i];
                if x + 1 < BOARD_SIZE && self[i + 1] == current {
                    return true;
                }
                if y + 1 < BOARD_SIZE && self[i + BOARD_SIZE] == current {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn compute_slide(&self, direction: Direction) -> SlideResult {
        let mut new_board = self.clone();
        let mut all_movements = Vec::new();
        let mut all_merge_dests = Vec::new();
        let mut total_score = 0u32;
        let mut changed = false;

        for i in 0..BOARD_SIZE {
            let indices = direction.line_indices(i);
            let line = indices.map(|idx| self[idx]);
            let (c, new_line, score, movements, merge_dests) =
                slide_line_with_movements(line, indices);

            all_movements.extend(movements);

            if c {
                changed = true;
                total_score += score;
                all_merge_dests.extend(merge_dests);
                for (idx, value) in indices.into_iter().zip(new_line) {
                    new_board[idx] = value;
                }
            }
        }

        SlideResult {
            changed,
            movements: all_movements,
            merge_destinations: all_merge_dests,
            new_board,
            score_gained: total_score,
        }
    }
}

fn slide_line_with_movements(
    line: [Option<NonZero<u8>>; BOARD_SIZE],
    indices: [usize; BOARD_SIZE],
) -> (
    bool,
    [Option<NonZero<u8>>; BOARD_SIZE],
    u32,
    Vec<SlideMovement>,
    Vec<usize>,
) {
    let tiles: Vec<(NonZero<u8>, usize)> = line
        .iter()
        .zip(indices.iter())
        .filter_map(|(cell, &idx)| cell.map(|v| (v, idx)))
        .collect();

    let mut result = [None; BOARD_SIZE];
    let mut score = 0u32;
    let mut movements = Vec::new();
    let mut merge_dests = Vec::new();
    let mut write = 0;
    let mut i = 0;

    while i < tiles.len() {
        let (val, orig_idx) = tiles[i];
        let dest = indices[write];

        if i + 1 < tiles.len() && tiles[i].0 == tiles[i + 1].0 {
            let (_, orig_idx2) = tiles[i + 1];
            let merged_exp = val.get() + 1;
            result[write] = Some(non_zero_exp(merged_exp));
            score += exp_to_value(merged_exp);

            movements.push(SlideMovement {
                from: orig_idx,
                to: dest,
            });
            movements.push(SlideMovement {
                from: orig_idx2,
                to: dest,
            });
            merge_dests.push(dest);

            i += 2;
        } else {
            result[write] = Some(val);
            movements.push(SlideMovement {
                from: orig_idx,
                to: dest,
            });
            i += 1;
        }
        write += 1;
    }

    (result != line, result, score, movements, merge_dests)
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.chunks(BOARD_SIZE).rev() {
            for cell in row {
                match cell {
                    Some(value) => write!(f, "{:6} ", exp_to_value(value.get()))?,
                    None => write!(f, "     . ")?,
                }
            }
            writeln!(f)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell_exp(exp: u8) -> Option<NonZero<u8>> {
        Some(non_zero_exp(exp))
    }

    fn board_with(entries: &[(usize, u8)]) -> Board {
        let mut board = Board::default();
        for &(index, exponent) in entries {
            board[index] = cell_exp(exponent);
        }
        board
    }

    #[test]
    fn slide_left_merges_once_for_three_equal_tiles() {
        let board = board_with(&[
            (Board::index(0, 0), 1),
            (Board::index(1, 0), 1),
            (Board::index(2, 0), 1),
        ]);

        let result = board.compute_slide(Direction::Left);

        assert!(result.changed);
        assert_eq!(result.score_gained, 4);
        assert_eq!(result.new_board[Board::index(0, 0)], cell_exp(2));
        assert_eq!(result.new_board[Board::index(1, 0)], cell_exp(1));
        assert_eq!(result.new_board[Board::index(2, 0)], None);
        assert_eq!(result.new_board[Board::index(3, 0)], None);
        assert_eq!(result.merge_destinations, vec![Board::index(0, 0)]);
    }

    #[test]
    fn slide_left_double_merge_for_four_equal_tiles() {
        let board = board_with(&[
            (Board::index(0, 1), 1),
            (Board::index(1, 1), 1),
            (Board::index(2, 1), 1),
            (Board::index(3, 1), 1),
        ]);

        let result = board.compute_slide(Direction::Left);

        assert!(result.changed);
        assert_eq!(result.score_gained, 8);
        assert_eq!(result.new_board[Board::index(0, 1)], cell_exp(2));
        assert_eq!(result.new_board[Board::index(1, 1)], cell_exp(2));
        assert_eq!(result.new_board[Board::index(2, 1)], None);
        assert_eq!(result.new_board[Board::index(3, 1)], None);
        assert_eq!(
            result.merge_destinations,
            vec![Board::index(0, 1), Board::index(1, 1)]
        );
    }

    #[test]
    fn slide_left_no_change_on_already_compacted_line() {
        let board = board_with(&[(Board::index(0, 0), 1), (Board::index(1, 0), 2)]);

        let result = board.compute_slide(Direction::Left);

        assert!(!result.changed);
        assert_eq!(result.score_gained, 0);
        assert_eq!(result.new_board[Board::index(0, 0)], cell_exp(1));
        assert_eq!(result.new_board[Board::index(1, 0)], cell_exp(2));
    }

    #[test]
    fn slide_vertical_moves_to_expected_edge() {
        let board = board_with(&[(Board::index(0, 0), 1), (Board::index(3, 3), 2)]);

        let up = board.compute_slide(Direction::Up);
        assert_eq!(up.new_board[Board::index(0, 3)], cell_exp(1));
        assert_eq!(up.new_board[Board::index(3, 3)], cell_exp(2));

        let down = board.compute_slide(Direction::Down);
        assert_eq!(down.new_board[Board::index(0, 0)], cell_exp(1));
        assert_eq!(down.new_board[Board::index(3, 0)], cell_exp(2));
    }

    #[test]
    fn can_move_true_when_board_has_empty_cell() {
        let board = board_with(&[(Board::index(0, 0), 1)]);
        assert!(board.can_move());
    }

    #[test]
    fn can_move_true_when_adjacent_equal_tiles_exist() {
        let board = board_with(&[
            (Board::index(0, 0), 1),
            (Board::index(1, 0), 1),
            (Board::index(2, 0), 2),
            (Board::index(3, 0), 3),
            (Board::index(0, 1), 4),
            (Board::index(1, 1), 5),
            (Board::index(2, 1), 6),
            (Board::index(3, 1), 7),
            (Board::index(0, 2), 8),
            (Board::index(1, 2), 9),
            (Board::index(2, 2), 10),
            (Board::index(3, 2), 11),
            (Board::index(0, 3), 12),
            (Board::index(1, 3), 13),
            (Board::index(2, 3), 14),
            (Board::index(3, 3), 15),
        ]);
        assert!(board.can_move());
    }

    #[test]
    fn can_move_false_when_board_is_full_and_blocked() {
        let board = board_with(&[
            (Board::index(0, 0), 1),
            (Board::index(1, 0), 2),
            (Board::index(2, 0), 3),
            (Board::index(3, 0), 4),
            (Board::index(0, 1), 5),
            (Board::index(1, 1), 6),
            (Board::index(2, 1), 7),
            (Board::index(3, 1), 8),
            (Board::index(0, 2), 9),
            (Board::index(1, 2), 10),
            (Board::index(2, 2), 11),
            (Board::index(3, 2), 12),
            (Board::index(0, 3), 13),
            (Board::index(1, 3), 14),
            (Board::index(2, 3), 15),
            (Board::index(3, 3), 16),
        ]);
        assert!(!board.can_move());
    }
}
