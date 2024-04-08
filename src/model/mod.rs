#![allow(dead_code)]

use std::{
    collections::HashMap,
    default,
    iter::zip,
    sync::mpsc::{Receiver, Sender},
    usize,
};

use egui::accesskit::VerticalOffset;
use log::{error, info};
use std::collections::hash_map;

use crate::ControlMsg;

struct Model {
    ui_sender: Sender<ControlMsg>,
    ui_reciever: Receiver<ControlMsg>,
    game: Game,
}
impl Model {
    fn new(send: Sender<ControlMsg>, recv: Receiver<ControlMsg>) -> Self {
        Model {
            ui_sender: send,
            ui_reciever: recv,
            game: Game::new(),
        }
    }
}

struct Game {
    board: Board,
    timer: ChessTimer,
    which_turn: Side,
}
impl Game {
    fn new() -> Self {
        let mut board: Board = Board::new();
        for col in 'a'..='e' {
            for row in 1..=8 {
                board.insert(
                    Position {
                        col,
                        row: row as usize,
                    },
                    None,
                );
            }
        }
        Game {
            board,
            timer: ChessTimer {},
            which_turn: Side::White,
        }
    }
}

enum Side {
    White,
    Black,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct Position {
    col: char,
    row: usize,
}
impl Position {
    fn get_vertical(&self) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        for row in 1..=8 {
            positions.push(Position { col: self.col, row });
        }
        positions
    }
    fn get_horizontal(&self) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        for col in 'a'..='e' {
            positions.push(Position { col, row: self.row });
        }
        positions
    }
    fn get_diagonals(&self) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();

        for (col, row) in zip(self.col..='e', self.row..=8) {
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
    fn get_offset(&self, row_offset: isize, col_offset: isize) -> Option<Position> {
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

        if new_col < 'a' || new_col > 'e' || new_row > 8 || new_row < 1 {
            None
        } else {
            Some(Position {
                row: new_row,
                col: new_col,
            })
        }
    }
    fn get_offsets(&self, offsets: Vec<(isize, isize)>) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        for (row_offset, col_offset) in offsets {
            if let Some(pos) = self.get_offset(row_offset, col_offset) {
                positions.push(pos);
            }
        }
        positions
    }
    fn get_adjacents(&self) -> Vec<Position> {
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

    fn get_knight_moves(&self) -> Vec<Position> {
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

    fn push_if_occupied(vec: &mut Vec<Position>, new_pos: Position, board: &Board) {
        if let Some(_) = board.get(&new_pos) {
            vec.push(new_pos)
        }
    }
}

fn push_if_exists<T>(vec: &mut Vec<T>, new_obj: Option<T>) {
    match new_obj {
        Some(t) => vec.push(t),
        None => (),
    }
}

type Board = HashMap<Position, Option<Piece>>;

#[derive(Default)]
struct ChessTimer {}

struct Piece {
    side: Side,
    piece_type: PieceType,
    current_pos: Position,
    has_moved: bool,
}

impl Piece {
    fn get_valid_moves(&self, board: &Board) -> Vec<Position> {
        let moves = match self.piece_type {
            PieceType::King => {
                let mut mvs = self.current_pos.get_adjacents();
                mvs.append(&mut self.get_available_castle_moves(board));
                mvs
            }
            PieceType::Queen => {
                let mut mvs = self.current_pos.get_horizontal();
                mvs.append(&mut self.current_pos.get_vertical());
                mvs.append(&mut self.current_pos.get_diagonals());
                mvs
            }

            PieceType::Rook => {
                let mut mvs = self.current_pos.get_horizontal();
                mvs.append(&mut self.current_pos.get_vertical());
                mvs
            }
            PieceType::Bishop => self.current_pos.get_diagonals(),
            PieceType::Knight => self.current_pos.get_knight_moves(),
            PieceType::Pawn => self.get_pawn_moves(board),
        };
        self.filter_blocked_moves(board, moves)
    }
    fn filter_check_moves(&self, board: &Board, moves: Vec<Position>) -> Vec<Position> {
        moves //TODO: This
    }
    fn filter_blocked_moves(&self, board: &Board, moves: Vec<Position>) -> Vec<Position> {
        moves
    }

    fn get_available_castle_moves(&self, board: &Board) -> Vec<Position> {
        Vec::new()
    }
    fn can_move_to(&self, to_pos: Position, board: &Board) -> bool {
        self.get_valid_moves(board).contains(&to_pos)
    }

    fn get_pawn_moves(&self, board: &Board) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        match self.side {
            Side::White => {
                if !self.has_moved {
                    positions.append(&mut self.current_pos.get_offsets(vec![(0, 2)]));
                };
                positions.append(&mut self.current_pos.get_offsets(vec![(0, 1)]));
                if let Some(up_left) = self.current_pos.get_offset(-1, 1) {
                    Position::push_if_occupied(&mut positions, up_left, board)
                }
                if let Some(up_right) = self.current_pos.get_offset(1, 1) {
                    Position::push_if_occupied(&mut positions, up_right, board)
                }
                positions
            }
            Side::Black => {
                if !self.has_moved {
                    positions.append(&mut self.current_pos.get_offsets(vec![(0, -2)]));
                };
                positions.append(&mut self.current_pos.get_offsets(vec![(0, -1)]));

                if let Some(down_left) = self.current_pos.get_offset(-1, -1) {
                    Position::push_if_occupied(&mut positions, down_left, board)
                }
                if let Some(down_right) = self.current_pos.get_offset(1, -1) {
                    Position::push_if_occupied(&mut positions, down_right, board)
                }
                positions
            }
        }
    }
}

enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}
impl PieceType {}

pub fn init_model(send: Sender<ControlMsg>, recv: Receiver<ControlMsg>) {
    let game = Model::new(send, recv);
    if let Err(e) = game.ui_sender.send(ControlMsg::Debug) {
        error!("{}", e)
    };
    while let Ok(msg) = game.ui_reciever.recv() {
        info!("Msg recieved: {}", msg)
    }
}
