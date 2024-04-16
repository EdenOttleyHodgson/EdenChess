#![allow(dead_code)]

use std::{
    collections::{HashMap, HashSet},
    default,
    iter::zip,
    sync::mpsc::{Receiver, Sender},
    usize,
};

use log::{error, info};
use std::collections::hash_map;

use crate::control::*;

struct Model {
    ui_sender: Sender<ModelMsg>,
    ui_reciever: Receiver<UiMsg>,
    game: Game,
}
impl Model {
    fn new(send: Sender<ModelMsg>, recv: Receiver<UiMsg>) -> Self {
        Model {
            ui_sender: send,
            ui_reciever: recv,
            game: Game::new(),
        }
    }
    fn model_loop(&self) {
        loop {
            match self.ui_reciever.recv() {
                Ok(m) => {
                    if let UiMsg::Quit = m {
                        break;
                    }
                    self.handle_message(m)
                }
                Err(e) => {
                    error!("{}", e);
                    break;
                }
            }
        }
    }
    fn handle_message(&self, msg: UiMsg) {}
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
    fn get_all_pieces_in(board: &Board, positions: Vec<Position>) -> Vec<&Piece> {
        let mut pieces: Vec<&Piece> = Vec::new();
        for pos in positions {
            if let Some(x) = board.get(&pos) {
                if let Some(piece) = x {
                    pieces.push(piece)
                }
            } else {
                error!("Position out of bounds!")
            }
        }
        pieces
    }
}

#[derive(PartialEq, Eq)]
enum Side {
    White,
    Black,
}

type Board = HashMap<Position, Option<Piece>>;

#[derive(Default)]
struct ChessTimer {}

pub struct Piece {
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
    fn filter_blocked_moves(&self, board: &Board, mut moves: Vec<Position>) -> Vec<Position> {
        let pieces = Game::get_all_pieces_in(board, moves);
        let friendly_piece_positions: Vec<Position> = pieces
            .iter()
            .filter(|x| x.side == self.side)
            .map(|x| x.current_pos)
            .collect();

        match self.piece_type {
            PieceType::Queen => {
                //get all pieces
                //remove all squares beyond piece

                let north_blocked: Vec<Position> = blocked_in(pieces, Direction::North);

                let east_blocked: Vec<Position> = blocked_in(pieces, Direction::East);

                let south_blocked: Vec<Position> = blocked_in(pieces, Direction::South);

                let west_blocked: Vec<Position> = blocked_in(pieces, Direction::West);

                let north_west_blocked: Vec<Position> = blocked_in(pieces, Direction::NorthWest);

                let north_east_blocked: Vec<Position> = blocked_in(pieces, Direction::NorthEast);

                let south_east_blocked: Vec<Position> = blocked_in(pieces, Direction::SouthEast);

                let south_west_blocked: Vec<Position> = blocked_in(pieces, Direction::SouthWest);

                let moves_to_remove: HashSet<&Position> = [
                    north_blocked,
                    north_east_blocked,
                    east_blocked,
                    south_east_blocked,
                    south_blocked,
                    south_west_blocked,
                    west_blocked,
                    north_east_blocked,
                    friendly_piece_positions,
                ]
                .concat()
                .iter()
                .collect();

                moves.retain(|m| !moves_to_remove.contains(m));
                moves
            }
            PieceType::Rook => {
                let pieces = Game::get_all_pieces_in(board, moves);

                let north_blocked: Vec<Position> = blocked_in(pieces, Direction::North);

                let east_blocked: Vec<Position> = blocked_in(pieces, Direction::East);

                let south_blocked: Vec<Position> = blocked_in(pieces, Direction::South);

                let west_blocked: Vec<Position> = blocked_in(pieces, Direction::West);

                let moves_to_remove: HashSet<&Position> = [
                    north_blocked,
                    east_blocked,
                    south_blocked,
                    west_blocked,
                    friendly_piece_positions,
                ]
                .concat()
                .iter()
                .collect();

                moves.retain(|m| !moves_to_remove.contains(m));
                moves
            }
            PieceType::Bishop => {
                let pieces = Game::get_all_pieces_in(board, moves);

                let north_west_blocked: Vec<Position> = blocked_in(pieces, Direction::NorthWest);

                let north_east_blocked: Vec<Position> = blocked_in(pieces, Direction::NorthEast);

                let south_east_blocked: Vec<Position> = blocked_in(pieces, Direction::SouthEast);

                let south_west_blocked: Vec<Position> = blocked_in(pieces, Direction::SouthWest);

                let moves_to_remove: HashSet<&Position> = [
                    north_east_blocked,
                    south_east_blocked,
                    south_west_blocked,
                    north_east_blocked,
                    friendly_piece_positions,
                ]
                .concat()
                .iter()
                .collect();

                moves.retain(|m| !moves_to_remove.contains(m));
                moves
            }
            PieceType::Pawn => {
                let pieces = Game::get_all_pieces_in(board, moves);
                let occupied: HashSet<Position> = pieces.iter().map(|x| x.current_pos).collect();
                let mut vert_moves = match self.side {
                    Side::White => self.current_pos.get_offsets(vec![(0, 1), (0, 2)]),
                    Side::Black => self.current_pos.get_offsets(vec![(0, -1), (0, -2)]),
                };
                vert_moves.retain(|x| !occupied.contains(x));
                moves.retain(|x| vert_moves.contains(x));
                moves
            }
            _ => {
                moves.retain(|x| !friendly_piece_positions.contains(x));
                moves
            }
        }
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

fn blocked_in(pieces: Vec<&Piece>, dir: Direction) -> Vec<Position> {
    pieces
        .iter()
        .flat_map(|p| p.current_pos.beyond(dir))
        .collect()
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

pub fn init_model(send: Sender<ModelMsg>, recv: Receiver<UiMsg>) {
    let game = Model::new(send, recv);
    if let Err(e) = game.ui_sender.send(ModelMsg::Debug("Started")) {
        error!("{}", e)
    };
    while let Ok(msg) = game.ui_reciever.recv() {
        info!("Msg recieved: {:?}", msg)
    }
}
