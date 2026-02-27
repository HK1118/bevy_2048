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

    // #[cfg(test)]
    // pub(super) fn slide(&mut self, direction: Direction) -> (bool, u32) {
    //     let mut changed = false;
    //     let mut total_score = 0u32;

    //     for i in 0..BOARD_SIZE {
    //         let indices = direction.line_indices(i);
    //         let line = indices.map(|index| self[index]);
    //         let (c, new_line, score) = slide_line(line);

    //         if c {
    //             changed = true;
    //             total_score += score;
    //             for (index, value) in indices.into_iter().zip(new_line) {
    //                 self[index] = value;
    //             }
    //         }
    //     }

    //     (changed, total_score)
    // }

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

// #[cfg(test)]
// fn slide_line(
//     line: [Option<NonZero<u8>>; BOARD_SIZE],
// ) -> (bool, [Option<NonZero<u8>>; BOARD_SIZE], u32) {
//     // パス1: Noneを除いた非空タイルをコンパクトに並べる
//     let tiles: Vec<NonZero<u8>> = line.into_iter().flatten().collect();

//     // パス2: 先頭から走査し、隣接する同値タイルをマージしながら result を構築する
//     let mut result = [None; BOARD_SIZE];
//     let mut score = 0u32;
//     let mut write = 0;
//     let mut i = 0;

//     while i < tiles.len() {
//         if i + 1 < tiles.len() && tiles[i] == tiles[i + 1] {
//             // 同値タイルを合体（指数+1 = 値×2）
//             let merged_exp = tiles[i].get() + 1;
//             result[write] = Some(non_zero_exp(merged_exp));
//             score += exp_to_value(merged_exp);
//             i += 2;
//         } else {
//             result[write] = Some(tiles[i]);
//             i += 1;
//         }
//         write += 1;
//     }

//     (result != line, result, score)
// }

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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn tile(v: u8) -> Option<NonZero<u8>> {
//         Some(non_zero_exp(v))
//     }

//     #[test]
//     fn test_slide_empty_line() {
//         let (changed, result, score) = slide_line([None; BOARD_SIZE]);
//         assert!(!changed);
//         assert_eq!(result, [None; BOARD_SIZE]);
//         assert_eq!(score, 0);
//     }

//     #[test]
//     fn test_slide_merge_two() {
//         // [2, 2, _, _] -> [4, _, _, _], score=4
//         let (changed, result, score) = slide_line([tile(1), tile(1), None, None]);
//         assert!(changed);
//         assert_eq!(result, [tile(2), None, None, None]);
//         assert_eq!(score, 4);
//     }

//     #[test]
//     fn test_slide_already_packed_no_change() {
//         // [2, 4, _, _] -> already packed, no change
//         let (changed, result, score) = slide_line([tile(1), tile(2), None, None]);
//         assert!(!changed);
//         assert_eq!(result, [tile(1), tile(2), None, None]);
//         assert_eq!(score, 0);
//     }

//     #[test]
//     fn test_slide_move_without_merge() {
//         // [_, 2, _, _] -> [2, _, _, _]
//         let (changed, result, score) = slide_line([None, tile(1), None, None]);
//         assert!(changed);
//         assert_eq!(result, [tile(1), None, None, None]);
//         assert_eq!(score, 0);
//     }

//     #[test]
//     fn test_slide_no_double_merge() {
//         // [2, 2, 2, 2] -> [4, 4, _, _], not [8, _, _, _]
//         let (changed, result, score) = slide_line([tile(1), tile(1), tile(1), tile(1)]);
//         assert!(changed);
//         assert_eq!(result, [tile(2), tile(2), None, None]);
//         assert_eq!(score, 8); // 4 + 4
//     }

//     #[test]
//     fn test_slide_three_same() {
//         // [2, 2, 2, _] -> [4, 2, _, _]
//         let (changed, result, score) = slide_line([tile(1), tile(1), tile(1), None]);
//         assert!(changed);
//         assert_eq!(result, [tile(2), tile(1), None, None]);
//         assert_eq!(score, 4);
//     }

//     #[test]
//     fn test_slide_merge_with_gap() {
//         // [2, _, 2, _] -> [4, _, _, _]
//         let (changed, result, score) = slide_line([tile(1), None, tile(1), None]);
//         assert!(changed);
//         assert_eq!(result, [tile(2), None, None, None]);
//         assert_eq!(score, 4);
//     }

//     #[test]
//     fn test_direction_indices() {
//         println!("Direction::Left: {:?}", Direction::Left.line_indices(1));
//         assert_eq!(Direction::Left.line_indices(0), [0, 1, 2, 3]);
//         assert_eq!(Direction::Right.line_indices(0), [3, 2, 1, 0]);
//         assert_eq!(Direction::Up.line_indices(0), [12, 8, 4, 0]);
//         assert_eq!(Direction::Down.line_indices(0), [0, 4, 8, 12]);
//     }
// }
