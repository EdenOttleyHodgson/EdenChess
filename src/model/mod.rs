#![allow(dead_code)]

use std::{
    collections::HashMap,
    default,
    iter::zip,
    sync::mpsc::{Receiver, Sender},
    usize,
};

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
    fn get_offset(&mut self, row_offset: isize, col_offset: isize) -> Option<Position> {
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
    fn get_offsets(&mut self, offsets: Vec<(isize, isize)>) -> Vec<Position> {
        let mut positions: Vec<Position> = Vec::new();
        for (row_offset, col_offset) in offsets {
            if let Some(pos) = self.get_offset(row_offset, col_offset) {
                positions.push(pos);
            }
        }
        positions
    }
    fn get_adjacents(&mut self) -> Vec<Position> {
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
}

type Board = HashMap<Position, Option<Piece>>;

#[derive(Default)]
struct ChessTimer {}

struct Piece {
    side: Side,
    piece_type: PieceType,
    current_pos: Position,
}

impl Piece {
    fn can_move_to(&self, to_pos: Position, board: &Board) -> bool {}
}

enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}
impl PieceType {
    fn get_valid_moves(&self, from_pos: Position, board: &Board) -> Vec<Position> {
        let moves = match self {
            PieceType::King => {}
            PieceType::Queen => todo!(),
            PieceType::Rook => todo!(),
            PieceType::Bishop => todo!(),
            PieceType::Knight => todo!(),
            PieceType::Pawn => todo!(),
        };
        let moves = self.filter_check_moves(board, moves);
        self.filter_blocked_moves(board, moves)
    }
    fn filter_check_moves(&self, board: &Board, moves: Vec<Position>) -> Vec<Position> {
        moves //TODO: This
    }
    fn filter_blocked_moves(&self, board: &Board, moves: Vec<Position>) -> Vec<Position> {
        moves
    }
}

pub fn init_model(send: Sender<ControlMsg>, recv: Receiver<ControlMsg>) {
    let game = Model::new(send, recv);
    if let Err(e) = game.ui_sender.send(ControlMsg::Debug) {
        error!("{}", e)
    };
    while let Ok(msg) = game.ui_reciever.recv() {
        info!("Msg recieved: {}", msg)
    }
}
