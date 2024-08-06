#![allow(dead_code)]

use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    iter::repeat,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Duration,
    usize,
};

use log::{debug, error, info, warn};
use ratatui::text::Text;

use crate::control::*;

type MoveList<'a> = Vec<(Piece, CBPosition)>;
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

    fn from_board_state(
        send: Sender<ModelMsg>,
        recv: Receiver<UiMsg>,
        board: Board,
        turn: Side,
    ) -> Self {
        Model {
            ui_sender: send,
            ui_reciever: recv,
            game: Game::from_board_state(board, turn),
        }
    }
    fn model_loop(&mut self) {
        loop {
            match self.ui_reciever.recv() {
                Ok(m) => {
                    if let UiMsg::Quit = m {
                        info!("Quit message recieved");
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
        info!("Model Loop Broken!")
    }
    fn handle_message(&mut self, msg: UiMsg) {
        if let UiMsg::GetBoardState = msg {
        } else {
            info!("Message recieved: {:?}", msg);
        }
        match msg {
            UiMsg::Debug(s) => debug!("debug message recieved: {}", s),
            UiMsg::CheckValidMove((from, to)) => todo!(),
            UiMsg::GetValidMoves(pos) => {
                let valid_moves = self.game.get_valid_moves(pos);
                debug!("valid moves : {:?}", valid_moves);
                if let Err(e) = self.ui_sender.send(ModelMsg::Moves(valid_moves)) {
                    error!("{}", e)
                };
            }
            UiMsg::MakeMove((from, to)) => {
                self.make_move(from, to);
            }
            UiMsg::GetBoardState => {
                self.ui_sender
                    .send(ModelMsg::BoardState(self.game.board.clone()));
            }
            UiMsg::Quit => unreachable!(),
        }
    }

    fn make_move(&mut self, from: CBPosition, to: CBPosition) {
        if let Some(_) = self
            .game
            .board
            .get(&from)
            .expect("From piece pos should be in bounds!")
        {
            info!("making move!");
            let mut new_board = self.simulate_move(from, to);
            let all_moves = get_all_moves(&new_board);
            if Self::check_move_is_valid(
                Self::get_king(&new_board, self.game.which_turn),
                &all_moves,
            ) && to
                != Self::get_king(&self.game.board, self.game.which_turn.flipped()).current_pos
            {
                info!("Move is valid!");
                if all_moves.len() == 0 {
                    let _ = self.ui_sender.send(ModelMsg::Stalemate);
                } else if self.check_for_checkmate(new_board.clone(), all_moves) {
                    let _ = self
                        .ui_sender
                        .send(ModelMsg::Checkmate(self.game.which_turn));
                };
                self.game.board = new_board;
            } else {
                info!("move is invalid!");
                let _ = self.ui_sender.send(ModelMsg::MoveIsInvalid);
                //send a message back to ui to play sound or whatever
            }
        } else {
            warn!("No piece at from pos despite move request")
        }
    }

    fn simulate_move(&self, from: CBPosition, to: CBPosition) -> Board {
        let mut new_board = self.game.board.clone();

        move_piece_simulating(&mut new_board, from, to, None);

        new_board
    }

    fn check_for_checkmate(&mut self, mut sim_board: Board, all_moves: MoveList) -> bool {
        debug!("Checking for checkmate");
        let enemy_turn = self.game.which_turn.flipped();
        debug!("{}", DebugBoard(&sim_board));
        if self.can_king_move(&mut sim_board, &all_moves, enemy_turn) {
            debug!("{}", DebugBoard(&sim_board));
            debug!("{all_moves:?}");
            debug!("King can move!");
            false
        } else {
            debug!("King cant move!");
            for (piece, to) in all_moves.into_iter().filter(|(p, _)| p.side == enemy_turn) {
                debug!("checking {piece:?} -> {to:?}");
                let original = piece.current_pos;
                let old_piece = move_piece_simulating(&mut sim_board, piece.current_pos, to, None);
                let king = Self::get_king(&sim_board, enemy_turn);
                let sim_moves = get_all_moves(&sim_board);
                if !Self::piece_under_attack(king, &sim_moves) {
                    debug!(
                        "{:?}",
                        sim_moves
                            .into_iter()
                            .filter(|(p, _)| p.piece_type == PieceType::Pawn)
                            .collect::<MoveList>()
                    );
                    debug!("{}", DebugBoard(&sim_board));
                    debug!("checkmate disproved!: {piece:?}->{to:?}");
                    return false;
                }
                move_piece_simulating(&mut sim_board, to, original, old_piece);
            }
            true
        }
    }

    fn check_move_is_valid(king: &Piece, all_moves: &MoveList) -> bool {
        //May add to this later
        !Self::piece_under_attack(king, all_moves)
    }

    fn can_king_move(&self, mut sim_board: &mut Board, all_moves: &MoveList, side: Side) -> bool {
        debug!("Checking if king can move");
        let king_pos = Self::get_king(sim_board, side).current_pos;
        all_moves
            .iter()
            .filter(|(p, _)| p.current_pos == king_pos)
            .filter(|(p, to)| {
                let old_piece = move_piece_simulating(&mut sim_board, p.current_pos, *to, None);
                // let new_all_moves = &get_all_moves(sim_board);
                let new_king = Self::get_king(sim_board, side);
                debug!("moves for {p:?} -> {to:?}: {:?}", get_all_moves(sim_board));
                let r = Self::piece_under_attack(new_king, &get_all_moves(sim_board));
                move_piece_simulating(&mut sim_board, *to, p.current_pos, old_piece);
                !r
            })
            .peekable()
            .peek()
            .inspect(|ms| debug!("king moves : {ms:?}"))
            .is_some()
    }

    fn piece_under_attack(piece: &Piece, all_moves: &MoveList) -> bool {
        all_moves
            .iter()
            .filter(|(_, pos)| *pos == piece.current_pos)
            .peekable()
            .peek()
            .inspect(|ms| debug!("{:?} under attack by {:?}", piece.current_pos, ms))
            .is_some()
    }
    // this needs to be called for every possible move
    fn is_state_in_checkmate(
        board: &Board,
        all_moves: &Vec<(&Piece, CBPosition)>,
        side: Side,
    ) -> bool {
        let king_pos = Self::get_king(board, side).current_pos;
        let king_under_attack = all_moves
            .iter()
            .filter(|(_, pos)| *pos == king_pos)
            .next()
            .is_some();
        let king_cant_move = all_moves
            .iter()
            .filter(|(p, _)| p.current_pos == king_pos)
            .next()
            .is_none();
        king_under_attack && king_cant_move
    }
    fn get_king(board: &Board, side: Side) -> &Piece {
        board
            .iter()
            .map(|(_, p)| p)
            .flatten()
            .find(|p| p.side == side && p.piece_type == PieceType::King)
            .expect("King should exist")
    }
}

struct Game {
    board: Board,
    timer: ChessTimer,
    which_turn: Side,
}
impl Game {
    fn new() -> Self {
        let mut board = board_setup();

        Game {
            board,
            timer: ChessTimer {},
            which_turn: Side::White,
        }
    }
    fn from_board_state(board: Board, turn: Side) -> Game {
        Game {
            board,
            timer: ChessTimer {},
            which_turn: turn,
        }
    }
    fn get_all_pieces_in<'a>(board: &'a Board, positions: &Vec<CBPosition>) -> Vec<&'a Piece> {
        let mut pieces: Vec<&Piece> = Vec::new();
        for pos in positions {
            if let Some(x) = board.get(&pos) {
                if let Some(piece) = x {
                    pieces.push(piece)
                }
            }
        }
        pieces
    }
    fn get_valid_moves(&self, moving_piece_pos: CBPosition) -> Vec<CBPosition> {
        if let Some(piece) = self
            .board
            .get(&moving_piece_pos)
            .expect("Position is out of bounds!")
        {
            piece.get_valid_moves(&self.board)
        } else {
            debug!("piece not in hashmap?");
            Vec::new()
        }
    }
}

struct DebugPositions(Vec<CBPosition>);
impl Display for DebugPositions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut write_string = String::new();
        for pos in self.0.iter() {
            write_string.push_str(&format!("{:?} | ", pos))
        }
        write!(f, "{}", write_string)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Side {
    White,
    Black,
}
impl Side {
    pub fn to_color(&self) -> ratatui::style::Color {
        match self {
            Side::White => ratatui::style::Color::White,
            Side::Black => ratatui::style::Color::Black,
        }
    }
    pub fn flip(&mut self) {
        match self {
            Side::White => *self = Side::Black,
            Side::Black => *self = Side::White,
        }
    }
    pub fn flipped(&self) -> Side {
        let mut temp = self.clone();
        temp.flip();
        temp
    }
}

impl From<Side> for String {
    fn from(value: Side) -> Self {
        match value {
            Side::White => "White".to_string(),
            Side::Black => "Black".to_string(),
        }
    }
}
impl<'a> Into<Text<'a>> for Side {
    fn into(self) -> Text<'a> {
        Text::from(String::from(self))
    }
}

struct DebugBoard<'a>(&'a Board);
impl<'a> Debug for DebugBoard<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut write_string = String::new();
        for (pos, piece) in self.0.iter() {
            if let Some(some_piece) = piece {
                write_string.push_str(&format!("{:?}:{} |", pos, some_piece));
            }
        }
        write!(f, "{}", write_string)
    }
}
impl<'a> Display for DebugBoard<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let board = &self.0;
        let mut fmt_string = String::from("\n");
        let fourty_one_underscores: String = repeat("_").take(41).collect();
        fmt_string.push_str(&fourty_one_underscores);
        fmt_string.push_str("\n");
        for row in (1..=8).rev() {
            let mut row_string = String::from("|");
            for col in ('a'..='h') {
                let pos = CBPosition { col, row };
                let piece = self.0.get(&pos).expect("Pos should be in bounds!");
                if let Some(p) = piece {
                    row_string = format!("{row_string}{:?}|", p)
                } else {
                    row_string = format!("{row_string}____|")
                }
            }

            // let row_string = board
            //     .iter()
            //     .filter(|(pos, _)| pos.row == row)
            //     .map(|(_, p)| p).collect().collect
            // .fold("|".to_owned(), |acc, p| {
            // if let Some(p) = p {
            // format!("{acc}{:?}|", p)
            // } else {
            // format!("{acc}____|")
            // }
            // })
            fmt_string.push_str(&row_string);
            fmt_string.push_str("\n")
        }
        write!(f, "{}", fmt_string)
    }
}
pub type Board = HashMap<CBPosition, Option<Piece>>;

fn move_piece(board: &mut Board, from: CBPosition, to: CBPosition) {
    if let Some(Some(mut from_piece)) = board.insert(from, None) {
        from_piece.current_pos = to;
        from_piece.has_moved = true;
        board.insert(to, Some(from_piece));
        // debug!("{}", DebugBoard(board));
    } else {
        error!("Trying to move nonexistant piece!")
    }
}

fn move_piece_simulating(
    board: &mut Board,
    from: CBPosition,
    to: CBPosition,
    replacement: Option<Piece>,
) -> Option<Piece> {
    let mut from_piece = board
        .insert(from, replacement)
        .expect("already checked")
        .expect("already checked"); //Evil
    from_piece.current_pos = to;

    board
        .insert(to, Some(from_piece))
        .expect("To should be in bounds!")
}

fn get_all_moves(board: &Board) -> MoveList {
    board
        .iter()
        .flat_map(|(_, p)| p)
        .flat_map(|p| {
            p.get_valid_moves(&board)
                .iter()
                .map(|m| (*p, *m))
                .collect::<Vec<(Piece, CBPosition)>>()
        })
        .collect()
}

fn board_setup() -> Board {
    let mut board = Board::new();
    for col in 'a'..='h' {
        for row in 1..=8 {
            board.insert(
                CBPosition {
                    col,
                    row: row as usize,
                },
                None,
            );
        }
    }
    //Insertion of predetermined pieces
    insert_piece(&mut board, 1, 'a', Side::White, PieceType::Rook);
    insert_piece(&mut board, 1, 'b', Side::White, PieceType::Knight);
    insert_piece(&mut board, 1, 'c', Side::White, PieceType::Bishop);
    insert_piece(&mut board, 1, 'd', Side::White, PieceType::Queen);
    insert_piece(&mut board, 1, 'e', Side::White, PieceType::King);
    insert_piece(&mut board, 1, 'f', Side::White, PieceType::Bishop);
    insert_piece(&mut board, 1, 'g', Side::White, PieceType::Knight);
    insert_piece(&mut board, 1, 'h', Side::White, PieceType::Rook);

    insert_piece(&mut board, 8, 'a', Side::Black, PieceType::Rook);
    insert_piece(&mut board, 8, 'b', Side::Black, PieceType::Knight);
    insert_piece(&mut board, 8, 'c', Side::Black, PieceType::Bishop);
    insert_piece(&mut board, 8, 'd', Side::Black, PieceType::Queen);
    insert_piece(&mut board, 8, 'e', Side::Black, PieceType::King);
    insert_piece(&mut board, 8, 'f', Side::Black, PieceType::Bishop);
    insert_piece(&mut board, 8, 'g', Side::Black, PieceType::Knight);
    insert_piece(&mut board, 8, 'h', Side::Black, PieceType::Rook);

    //insertion of pawns

    for row in 'a'..='h' {
        insert_piece(&mut board, 2, row, Side::White, PieceType::Pawn);
        insert_piece(&mut board, 7, row, Side::Black, PieceType::Pawn);
    }

    board
}

fn empty_board() -> Board {
    let mut board = Board::new();
    for col in 'a'..='h' {
        for row in 1..=8 {
            board.insert(
                CBPosition {
                    col,
                    row: row as usize,
                },
                None,
            );
        }
    }
    board
}

fn insert_piece(board: &mut Board, row: usize, col: char, side: Side, piece_type: PieceType) {
    let pos = CBPosition { col, row };
    let piece = Piece::new(side, piece_type, pos);
    board.insert(pos, Some(piece));
}

#[derive(Default)]
struct ChessTimer {}

#[derive(Clone, Copy)]
pub struct Piece {
    pub side: Side,
    pub piece_type: PieceType,
    current_pos: CBPosition,
    has_moved: bool,
}
impl Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut write_string = String::new();
        match self.side {
            Side::White => write_string.push_str("w"),
            Side::Black => write_string.push_str("b"),
        }
        write_string.push_str(&format!("{}", self));
        write_string.push_str(&format!("{:?}", self.current_pos));
        match self.piece_type {
            PieceType::Pawn => {
                if self.has_moved {
                    write_string.push_str("t")
                }
            }
            _ => (),
        }

        write!(f, "{}", write_string)
    }
}
impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_str = match self.piece_type {
            PieceType::King => "K",
            PieceType::Queen => "Q",
            PieceType::Rook => "R",
            PieceType::Bishop => "B",
            PieceType::Knight => "N",
            PieceType::Pawn => "P",
        };
        write!(f, "{}", display_str)
    }
}

impl Piece {
    fn new(side: Side, piece_type: PieceType, current_pos: CBPosition) -> Piece {
        Piece {
            side,
            piece_type,
            current_pos,
            has_moved: false,
        }
    }

    fn get_valid_moves(&self, board: &Board) -> Vec<CBPosition> {
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
        let moves = self.filter_blocked_moves(board, moves);
        moves
    }
    fn filter_blocked_moves(&self, board: &Board, mut moves: Vec<CBPosition>) -> Vec<CBPosition> {
        let pieces = Game::get_all_pieces_in(board, &moves);
        let friendly_piece_positions: Vec<CBPosition> = pieces
            .iter()
            .filter(|x| x.side == self.side)
            .map(|x| x.current_pos)
            .collect();

        let mut moves = match self.piece_type {
            PieceType::Queen => self.filter_queen_moves(pieces, friendly_piece_positions, moves),
            PieceType::Rook => self.filter_rook_moves(board, moves, friendly_piece_positions),
            PieceType::Bishop => self.filter_bishop_moves(board, moves, friendly_piece_positions),
            PieceType::Pawn => self.filter_pawn_moves(board, moves),
            _ => {
                moves.retain(|x| !friendly_piece_positions.contains(x));
                moves
            }
        };
        // moves.retain(|x| *x != Model::get_king(board, self.side.flipped()).current_pos);
        moves
    }

    fn filter_pawn_moves(
        &self,
        board: &HashMap<CBPosition, Option<Piece>>,
        mut moves: Vec<CBPosition>,
    ) -> Vec<CBPosition> {
        let pieces = Game::get_all_pieces_in(board, &moves);
        let occupied: HashSet<CBPosition> = pieces.iter().map(|x| x.current_pos).collect();
        let offsets = if self.has_moved {
            vec![(1, 0)]
        } else {
            vec![(1, 0), (2, 0)]
        };
        let offsets = match self.side {
            Side::White => offsets,
            Side::Black => offsets.iter().map(|x| (x.0 * -1, x.1)).collect(),
        };

        let mut vert_moves = self.current_pos.get_offsets(offsets);
        vert_moves.retain(|x| !occupied.contains(x));
        moves.retain(|x| vert_moves.contains(x));
        let diag_squares = match self.side {
            Side::White => self.current_pos.get_offsets(vec![(1, 1), (1, -1)]),
            Side::Black => self.current_pos.get_offsets(vec![(-1, 1), (-1, -1)]),
        };
        for square in diag_squares {
            if let Some(p) = board.get(&square).expect("Pos should exist in board!") {
                if p.side != self.side {
                    moves.push(square)
                }
            }
        }
        moves
    }

    fn filter_bishop_moves(
        &self,
        board: &HashMap<CBPosition, Option<Piece>>,
        mut moves: Vec<CBPosition>,
        friendly_piece_positions: Vec<CBPosition>,
    ) -> Vec<CBPosition> {
        let pieces = Game::get_all_pieces_in(board, &moves);

        let north_west_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::NorthWest);

        let north_east_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::NorthEast);

        let south_east_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::SouthEast);

        let south_west_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::SouthWest);

        let moves_to_remove = [
            north_east_blocked,
            south_east_blocked,
            south_west_blocked,
            north_west_blocked,
            friendly_piece_positions,
        ]
        .concat();
        let moves_to_remove: HashSet<&CBPosition> = moves_to_remove.iter().collect();

        moves.retain(|m| !moves_to_remove.contains(m));
        moves
    }

    fn filter_rook_moves(
        &self,
        board: &HashMap<CBPosition, Option<Piece>>,
        mut moves: Vec<CBPosition>,
        friendly_piece_positions: Vec<CBPosition>,
    ) -> Vec<CBPosition> {
        let pieces = Game::get_all_pieces_in(board, &moves);

        let north_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::North);

        let east_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::East);

        let south_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::South);

        let west_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::West);

        let moves_to_remove = [
            north_blocked,
            east_blocked,
            south_blocked,
            west_blocked,
            friendly_piece_positions,
        ]
        .concat();

        let moves_to_remove: HashSet<&CBPosition> = moves_to_remove.iter().collect();

        moves.retain(|m| !moves_to_remove.contains(m));
        moves
    }

    fn filter_queen_moves(
        &self,
        pieces: Vec<&Piece>,
        friendly_piece_positions: Vec<CBPosition>,
        mut moves: Vec<CBPosition>,
    ) -> Vec<CBPosition> {
        let north_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::North);

        let east_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::East);

        let south_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::South);

        let west_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::West);

        let north_west_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::NorthWest);

        let north_east_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::NorthEast);

        let south_east_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::SouthEast);

        let south_west_blocked: Vec<CBPosition> = self.blocked_in(&pieces, Direction::SouthWest);

        let moves_to_remove = [
            north_blocked,
            north_east_blocked,
            east_blocked,
            south_east_blocked,
            south_blocked,
            south_west_blocked,
            west_blocked,
            north_west_blocked,
            friendly_piece_positions,
        ]
        .concat();

        let moves_to_remove: HashSet<&CBPosition> = moves_to_remove.iter().collect();

        moves.retain(|m| !moves_to_remove.contains(m));
        moves
    }
    fn get_available_castle_moves(&self, board: &Board) -> Vec<CBPosition> {
        Vec::new()
    }
    fn can_move_to(&self, to_pos: CBPosition, board: &Board) -> bool {
        self.get_valid_moves(board).contains(&to_pos)
    }

    fn get_pawn_moves(&self, board: &Board) -> Vec<CBPosition> {
        debug!("pawn moves board: {}", DebugBoard(board));
        let mut positions: Vec<CBPosition> = Vec::new();
        match self.side {
            Side::White => {
                if !self.has_moved {
                    positions.append(&mut self.current_pos.get_offsets(vec![(2, 0)]));
                };
                positions.append(&mut self.current_pos.get_offsets(vec![(1, 0)]));
                if let Some(up_left) = self.current_pos.get_offset(1, -1) {
                    CBPosition::push_if_occupied(&mut positions, up_left, board)
                }
                if let Some(up_right) = self.current_pos.get_offset(1, 1) {
                    CBPosition::push_if_occupied(&mut positions, up_right, board)
                }
                debug!("pawn moves: {:?} -> {positions:?}", self.current_pos);
                positions
            }
            Side::Black => {
                if !self.has_moved {
                    positions.append(&mut self.current_pos.get_offsets(vec![(-2, 0)]));
                };
                positions.append(&mut self.current_pos.get_offsets(vec![(-1, 0)]));

                if let Some(down_left) = self.current_pos.get_offset(-1, -1) {
                    CBPosition::push_if_occupied(&mut positions, down_left, board)
                }
                if let Some(down_right) = self.current_pos.get_offset(-1, 1) {
                    CBPosition::push_if_occupied(&mut positions, down_right, board)
                }
                positions
            } // she en on my passant til i yeah
        }
    }
    fn blocked_in(&self, pieces: &Vec<&Piece>, dir: Direction) -> Vec<CBPosition> {
        let closest_pos = pieces
            .iter()
            .filter(|x| self.current_pos.beyond(dir).contains(&x.current_pos))
            .map(|x| x.current_pos)
            .min_by(|x, y| {
                self.current_pos
                    .positional_difference(x)
                    .cmp(&self.current_pos.positional_difference(y))
            });
        let o = match closest_pos {
            Some(p) => p.beyond(dir),
            None => Vec::<CBPosition>::new(),
        };
        o
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}
impl PieceType {}

pub fn init_model(send: Sender<ModelMsg>, recv: Receiver<UiMsg>) {
    let mut model = Model::new(send, recv);
    if let Err(e) = model.ui_sender.send(ModelMsg::Debug("Started")) {
        error!("{}", e)
    };
    model.model_loop();

    info!("loop broken: model thread ending");
}

#[cfg(test)]
mod tests {

    use core::panic;
    use std::{future, thread::JoinHandle};

    use lazy_static::lazy_static;

    use crate::model::*;
    lazy_static! {
        static ref CHECKMATE_BOARDS: Vec<(&'static str, Board, (CBPosition, CBPosition))> = {
            use crate::model::{PieceType::*, Side::*};
            let mut checkmate_boards: Vec<(&'static str, Board, (CBPosition, CBPosition))> =
                Vec::new();

            let mut anastasias_mate = empty_board();
            insert_piece(&mut anastasias_mate, 1, 'g', White, King);
            insert_piece(&mut anastasias_mate, 3, 'e', White, Rook);
            insert_piece(&mut anastasias_mate, 7, 'e', White, Knight);
            insert_piece(&mut anastasias_mate, 7, 'g', Black, Pawn);
            insert_piece(&mut anastasias_mate, 8, 'h', Black, King);
            let mv = (CBPosition::from("e3"), CBPosition::from("h3"));
            checkmate_boards.push(("Anastasia's Mate", anastasias_mate, mv));

            let mut anderssens_mate = empty_board();
            insert_piece(&mut anderssens_mate, 8, 'g', Side::Black, PieceType::King);
            insert_piece(&mut anderssens_mate, 6, 'f', Side::White, PieceType::King);
            insert_piece(&mut anderssens_mate, 7, 'g', Side::White, PieceType::Pawn);
            insert_piece(&mut anderssens_mate, 2, 'h', Side::White, PieceType::Rook);
            let mv = (CBPosition::from("h2"), CBPosition::from("h8"));
            checkmate_boards.push(("Anderssen's Mate", anderssens_mate, mv));

            let mut arabian_mate = empty_board();
            insert_piece(&mut arabian_mate, 7, 'b', White, Rook);
            insert_piece(&mut arabian_mate, 6, 'f', White, Knight);
            insert_piece(&mut arabian_mate, 1, 'g', White, King);
            insert_piece(&mut arabian_mate, 8, 'h', Black, King);
            let mv = (CBPosition::from("b7"), CBPosition::from("h7"));
            checkmate_boards.push(("Arabian Mate", arabian_mate, mv));

            let mut balestra_mate = empty_board();
            insert_piece(&mut balestra_mate, 1, 'g', White, King);
            insert_piece(&mut balestra_mate, 3, 'f', White, Bishop);
            insert_piece(&mut balestra_mate, 6, 'f', White, Queen);
            insert_piece(&mut balestra_mate, 8, 'e', Black, King);
            let mv = (CBPosition::from("f3"), CBPosition::from("c6"));
            checkmate_boards.push(("Balestra Mate", balestra_mate, mv));

            checkmate_boards
        };
    }

    #[test]
    fn checkmate() {
        crate::test_init();
        let mut threads: Vec<JoinHandle<Result<(), &'static str>>> = Vec::new();
        let mut fails: Vec<&'static str> = Vec::new();
        for mate in CHECKMATE_BOARDS.clone().into_iter() {
            threads.push(thread::spawn(|| test_individual_checkmate(mate)));
        }
        for thread in threads {
            let res = thread.join().unwrap();
            match res {
                Ok(_) => (),
                Err(name) => fails.push(name),
            }
        }
        if !fails.is_empty() {
            let mut panic_str = String::from("Checkmates Failed:\n");
            for fail in fails {
                panic_str.push_str(&format!("     {fail}\n"));
            }
            panic!("{panic_str}")
        }
    }

    fn test_individual_checkmate(
        (name, board, mv): (&'static str, Board, (CBPosition, CBPosition)),
    ) -> Result<(), &'static str> {
        let (model_send, model_recv) = channel();
        let (ui_send, ui_recv) = channel();

        let mut model = Model::from_board_state(model_send, ui_recv, board, Side::White);
        thread::spawn(move || model.model_loop());
        ui_send.send(UiMsg::MakeMove(mv)).unwrap();
        thread::sleep(Duration::from_secs(3));
        model_recv
            .try_iter()
            .find(|msg| *msg == ModelMsg::Checkmate(Side::White))
            .map(|_| ())
            .ok_or(name)
    }
}
