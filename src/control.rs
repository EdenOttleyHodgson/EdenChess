use std::{collections::HashMap, iter::zip};

use crate::model::Piece;
use egui::Direction as glog;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Position {
    pub col: char,
    pub row: usize,
}
type Board = HashMap<Position, Option<Piece>>;
impl Position {
    pub fn get_vertical(&self) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        for row in 1..=8 {
            positions.push(Position { col: self.col, row });
        }
        positions
    }
    pub fn get_horizontal(&self) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        for col in 'a'..='h' {
            positions.push(Position { col, row: self.row });
        }
        positions
    }
    pub fn get_diagonals(&self) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();

        for (col, row) in zip(self.col..='h', self.row..=8) {
            positions.push(Position { col, row });
        } //up-right diagonal

        for (col, row) in zip(self.col..='a', 1..=self.row) {
            positions.push(Position { col, row });
        } //down-right

        for (col, row) in zip('a'..=self.col, 1..=self.row) {
            positions.push(Position { col, row });
        } //down-left

        for (col, row) in zip('a'..=self.col, self.row..=8) {
            positions.push(Position { col, row });
        } //up-left
        positions
    }
    pub fn get_offset(&self, row_offset: isize, col_offset: isize) -> Option<Position> {
        let new_row = if row_offset < 0 {
            let row_offset = row_offset.abs() as usize;
            self.row - row_offset
        } else {
            self.row - row_offset as usize
        };

        let new_col = if col_offset < 0 {
            let col_offset = row_offset.abs() as u8;
            (self.col as u8 - col_offset) as char
        } else {
            (self.col as u8 + col_offset as u8) as char
        };

        if new_col < 'a' || new_col > 'h' || new_row > 8 || new_row < 1 {
            None
        } else {
            Some(Position {
                row: new_row,
                col: new_col,
            })
        }
    }
    pub fn get_offsets(&self, offsets: Vec<(isize, isize)>) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        for (row_offset, col_offset) in offsets {
            if let Some(pos) = self.get_offset(row_offset, col_offset) {
                positions.push(pos);
            }
        }
        positions
    }
    pub fn get_adjacents(&self) -> Vec<Position> {
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

    pub fn get_knight_moves(&self) -> Vec<Position> {
        let offsets = vec![
            (2, 1),
            (1, 2),
            (-2, 1),
            (-1, 2),
            (-2, -1),
            (-1, -2),
            (2, -1),
            (1, -2),
        ];
        self.get_offsets(offsets)
    }

    pub fn push_if_occupied(vec: &mut Vec<Position>, new_pos: Position, board: &Board) {
        if let Some(_) = board.get(&new_pos) {
            vec.push(new_pos)
        }
    }
    pub fn beyond(&self, dir: Direction) -> Vec<Position> {
        match dir {
            Direction::North => Self::north_of(&self),
            Direction::NorthEast => {
                let mut ps = Self::north_of(&self);
                ps.append(&mut Self::east_of(&self));
                ps
            }
            Direction::East => Self::east_of(&self),
            Direction::SouthEast => {
                let mut ps = Self::south_of(&self);
                ps.append(&mut Self::east_of(&self));
                ps
            }
            Direction::South => Self::south_of(&self),
            Direction::SouthWest => {
                let mut ps = Self::south_of(&self);
                ps.append(&mut Self::west_of(&self));
                ps
            }
            Direction::West => Self::west_of(&self),
            Direction::NorthWest => {
                let mut ps = Self::north_of(&self);
                ps.append(&mut Self::west_of(&self));
                ps
            }
        }
    }
    pub fn east_of(pos: &Position) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        let start_bound = (pos.col as u8 + 1) as char;
        for col in start_bound..='h' {
            positions.push(Position { col, row: pos.row })
        }
        positions
    }
    pub fn north_of(pos: &Position) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        for row in (pos.row + 1)..=8 {
            positions.push(Position { col: pos.col, row })
        }
        positions
    }
    pub fn west_of(pos: &Position) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        for row in 1..pos.row {
            positions.push(Position { col: pos.col, row })
        }
        positions
    }
    pub fn south_of(pos: &Position) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        for col in 'a'..pos.col {
            positions.push(Position { col, row: pos.row })
        }
        positions
    }
}
#[derive(Clone, Copy)]
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
    pub fn relative_direction(anchor: Position, relative: Position) -> Direction {
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
    CheckValidMove((Position, Position)),
    GetValidMoves(Position),
    MakeMove((Position, Position)),
    Quit,
}

#[derive(Debug)]

pub enum ModelMsg {
    Debug(&'static str),
    MoveIsValid((Position, Position)),
    Moves(Position, Vec<Position>),
}
