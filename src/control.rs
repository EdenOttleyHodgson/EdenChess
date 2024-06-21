use std::{
    collections::HashMap,
    fmt::{Debug, Write},
    isize,
    iter::zip,
    ops::{Add, Sub},
};

use log::{debug, warn};
use ratatui::layout::Positions;

use crate::model::Piece;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct CBPosition {
    pub col: char,
    pub row: usize,
}
impl Debug for CBPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = format!("{}{}", self.col, self.row);
        f.write_str(&out)
    }
}
type Board = HashMap<CBPosition, Option<Piece>>;
impl CBPosition {
    pub fn get_vertical(&self) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        for row in 1..=8 {
            positions.push(CBPosition { col: self.col, row });
        }
        positions = positions.into_iter().filter(|x| x != self).collect();
        positions
    }
    pub fn get_horizontal(&self) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        for col in 'a'..='h' {
            positions.push(CBPosition { col, row: self.row });
        }
        positions = positions.into_iter().filter(|x| x != self).collect();
        positions
    }
    pub fn get_diagonals(&self) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        positions.append(&mut self.beyond(Direction::NorthEast));
        positions.append(&mut self.beyond(Direction::NorthWest));
        positions.append(&mut self.beyond(Direction::SouthEast));
        positions.append(&mut self.beyond(Direction::SouthWest));
        positions = positions.into_iter().filter(|x| x != self).collect();
        positions
    }
    pub fn get_offset(&self, row_offset: isize, col_offset: isize) -> Option<CBPosition> {
        let new_row = if row_offset < 0 {
            self.row as isize - row_offset.abs()
        } else {
            self.row as isize + row_offset
        };

        let new_col = if col_offset < 0 {
            self.col as isize - col_offset.abs()
        } else {
            self.col as isize + col_offset
        };

        let new_col = if new_col > 0 && new_col < 256 {
            new_col as u8 as char
        } else {
            return None;
        };

        if new_col < 'a' || new_col > 'h' || new_row > 8 || new_row < 1 {
            debug!(
                "Returning none from get offset, origin:{:?}, offset:{},{}, new_col:{}, new_row:{}",
                self, row_offset, col_offset, new_col, new_row
            );
            None
        } else {
            Some(CBPosition {
                row: new_row as usize,
                col: new_col as u8 as char,
            })
        }
    }
    pub fn get_offsets(&self, offsets: Vec<(isize, isize)>) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        for (row_offset, col_offset) in offsets.iter() {
            if let Some(pos) = self.get_offset(*row_offset, *col_offset) {
                positions.push(pos);
            }
        }
        debug!("offsets: {:?} -> positions: {:?}", offsets, positions);
        positions
    }
    pub fn get_adjacents(&self) -> Vec<CBPosition> {
        let offsets = vec![
            (1, 0),
            (0, 1),
            (1, 1),
            (-1, 0),
            (0, -1),
            (-1, -1),
            (-1, 1),
            (1, -1),
        ];
        self.get_offsets(offsets)
    }

    pub fn get_knight_moves(&self) -> Vec<CBPosition> {
        let offsets = vec![
            (1, 2),
            (2, 1),
            (2, -1),
            (1, -2),
            (-2, -1),
            (-1, -2),
            (-1, 2),
            (-2, 1),
        ];
        let r = self.get_offsets(offsets);
        debug!("result of get knight moves: {:?}", r);
        r
    }

    pub fn push_if_occupied(vec: &mut Vec<CBPosition>, new_pos: CBPosition, board: &Board) {
        if let Some(_) = board.get(&new_pos) {
            vec.push(new_pos)
        }
    }
    pub fn beyond(&self, dir: Direction) -> Vec<CBPosition> {
        match dir {
            Direction::North => Self::north_of(&self),
            Direction::NorthEast => Self::northeast_of(&self),
            Direction::East => Self::east_of(&self),
            Direction::SouthEast => Self::southeast_of(&self),
            Direction::South => Self::south_of(&self),
            Direction::SouthWest => Self::southwest_of(&self),
            Direction::West => Self::west_of(&self),
            Direction::NorthWest => Self::northwest_of(&self),
        }
    }
    pub fn east_of(pos: &CBPosition) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        let start_bound = (pos.col as u8 + 1) as char;
        for col in start_bound..='h' {
            positions.push(CBPosition { col, row: pos.row })
        }
        positions
    }
    pub fn north_of(pos: &CBPosition) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        for row in (pos.row + 1)..=8 {
            positions.push(CBPosition { col: pos.col, row })
        }
        positions
    }
    pub fn south_of(pos: &CBPosition) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        for row in 1..pos.row {
            positions.push(CBPosition { col: pos.col, row })
        }
        positions
    }
    pub fn west_of(pos: &CBPosition) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        for col in 'a'..pos.col {
            positions.push(CBPosition { col, row: pos.row })
        }
        positions
    }
    pub fn northeast_of(pos: &CBPosition) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        let mut cur_pos = *pos;

        while let Some(new_pos) = cur_pos.get_offset(1, 1) {
            positions.push(new_pos);
            cur_pos = new_pos;
        }
        positions
    }
    pub fn southeast_of(pos: &CBPosition) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        let mut cur_pos = *pos;

        while let Some(new_pos) = cur_pos.get_offset(1, -1) {
            positions.push(new_pos);
            cur_pos = new_pos;
        }

        positions
    }
    pub fn northwest_of(pos: &CBPosition) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        let mut cur_pos = *pos;
        while let Some(new_pos) = cur_pos.get_offset(-1, 1) {
            positions.push(new_pos);
            cur_pos = new_pos;
        }
        positions
    }
    pub fn southwest_of(pos: &CBPosition) -> Vec<CBPosition> {
        let mut positions: Vec<CBPosition> = Vec::new();
        let mut cur_pos = *pos;
        while let Some(new_pos) = cur_pos.get_offset(-1, -1) {
            positions.push(new_pos);
            cur_pos = new_pos;
        }
        positions
    }
    pub fn move_cursor_right(&mut self) {
        let new_col = char_add(self.col, 1);
        if new_col > 'h' {
            warn!("Cursor trying to escape right!");
            return;
        };
        self.col = new_col
    }
    pub fn move_cursor_left(&mut self) {
        let new_col = char_sub(self.col, 1);
        if new_col < 'a' {
            warn!("Cursor trying to escape left!");
            return;
        };
        self.col = new_col
    }
    pub fn move_cursor_down(&mut self) {
        if self.row == 1 {
            warn!("Cursor trying to escape up!");
            return;
        }
        self.row = self.row - 1
    }
    pub fn move_cursor_up(&mut self) {
        if self.row == 8 {
            warn!("Cursor trying to escape down!");
            return;
        }
        self.row = self.row + 1
    }
    pub fn positional_difference(&self, other: &CBPosition) -> usize {
        let num_self_col = self.col as isize;
        let num_other_col = other.col as isize;
        let col_diff = num_self_col - num_other_col;
        let row_diff = self.row as isize - other.row as isize;
        (col_diff.abs() + row_diff.abs()) as usize
    }
}
impl Sub for CBPosition {
    type Output = Option<CBPosition>;

    fn sub(self, rhs: Self) -> Self::Output {
        let num_self_col = self.col as u8;
        let num_rhs_col = rhs.col as u8;
        let col = if num_rhs_col > num_self_col {
            return None;
        } else {
            (num_self_col - num_rhs_col) as char
        };
        let row = if rhs.row > self.row {
            return None;
        } else {
            self.row - rhs.row
        };
        Some(CBPosition { col, row })
    }
}
impl Add for CBPosition {
    type Output = Option<CBPosition>;

    fn add(self, rhs: Self) -> Self::Output {
        let num_self_col = self.col as u8;
        let num_rhs_col = rhs.col as u8;
        let col = if (num_rhs_col + num_self_col) > 8 {
            return None;
        } else {
            (num_self_col + num_rhs_col) as char
        };
        let row = if (rhs.row + self.row) > 8 {
            return None;
        } else {
            self.row + rhs.row
        };
        Some(CBPosition { col, row })
    }
}
// impl PartialOrd for CBPosition {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         let self_magnitude = (((self.col as usize) ^ 2 + (self.row ^ 2)) as f32).sqrt();
//         let other_magnitude = (((other.col as usize) ^ 2 + (other.row ^ 2)) as f32).sqrt();
//         if self_magnitude > other_magnitude {
//             Some(std::cmp::Ordering::Greater)
//         } else if self_magnitude < other_magnitude {
//             Some(std::cmp::Ordering::Less)
//         } else {
//             Some(std::cmp::Ordering::Equal)
//         }
//     }
// }
// impl Ord for CBPosition {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.partial_cmp(other)
//             .expect("Partial Comp of CBPositions always succeeds!")
//     }
// }

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

enum UpDown {
    Up,
    Down,
    Same,
}
enum LeftRight {
    Left,
    Right,
    Same,
}

impl Direction {
    pub fn relative_direction(anchor: CBPosition, relative: CBPosition) -> Direction {
        let left_right = if anchor.col > relative.col {
            LeftRight::Left
        } else if anchor.col < relative.col {
            LeftRight::Right
        } else {
            LeftRight::Same
        };
        let up_down = if anchor.row > relative.row {
            UpDown::Down
        } else if anchor.row < relative.row {
            UpDown::Up
        } else {
            UpDown::Same
        };
        match up_down {
            UpDown::Up => match left_right {
                LeftRight::Left => Direction::NorthWest,
                LeftRight::Right => Direction::NorthEast,
                LeftRight::Same => Direction::North,
            },
            UpDown::Down => match left_right {
                LeftRight::Left => Direction::SouthWest,
                LeftRight::Right => Direction::SouthEast,
                LeftRight::Same => Direction::South,
            },
            UpDown::Same => match left_right {
                LeftRight::Left => Direction::West,
                LeftRight::Right => Direction::East,
                LeftRight::Same => panic!("The two positions should not be equal!"),
            },
        }
    }
}

pub fn push_if_exists<T>(vec: &mut Vec<T>, new_obj: Option<T>) {
    match new_obj {
        Some(t) => vec.push(t),
        None => (),
    }
}

#[derive(Debug)]
pub enum UiMsg {
    Debug(&'static str),
    CheckValidMove((CBPosition, CBPosition)),
    GetValidMoves(CBPosition),
    MakeMove((CBPosition, CBPosition)),
    GetBoardState,
    Quit,
}

#[derive(Debug)]
pub enum ModelMsg {
    Debug(&'static str),
    MoveIsValid((CBPosition, CBPosition)),
    Moves(Vec<CBPosition>),
    BoardState(Board),
}

pub fn char_add(c: char, i: u8) -> char {
    ((c as u8) + i) as char
}

pub fn char_sub(c: char, i: u8) -> char {
    ((c as u8) - i) as char
}
