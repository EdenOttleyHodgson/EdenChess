#![allow(dead_code)]

use std::{
    collections::HashMap,
    default,
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
                board.insert(Position(col, row as usize), None);
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

#[derive(PartialEq, Eq, Hash)]
struct Position(char, usize);

type Board = HashMap<Position, Option<Piece>>;

#[derive(Default)]
struct ChessTimer {}

struct Piece {
    side: Side,
    piece_type: PieceType,
}

enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
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
