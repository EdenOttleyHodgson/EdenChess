use std::{
    collections::HashMap,
    io::{self, Result},
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
};
use log::*;

use crate::{
    control::{CBPosition, ModelMsg, UiMsg},
    model::{Board, Piece, PieceType, Side},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph},
};

use self::{chessboard::Chessboard, infobox::Infobox};
mod chessboard;
mod infobox;
pub mod tui;

pub fn init_ui(send: Sender<UiMsg>, recv: Receiver<ModelMsg>) -> io::Result<()> {
    let mut terminal = tui::init()?;
    let mut eden_chess_ui = EdenChessUi {
        send,
        recv,
        exit: false,
        board: None,
        cursor: CBPosition { col: 'a', row: 1 },
        square_selected: None,
        valid_moves: None,
        game_data: GameData::new(),
    };
    eden_chess_ui.run(&mut terminal)?;
    tui::restore()?;
    Ok(())
}

struct EdenChessUi {
    send: Sender<UiMsg>,
    recv: Receiver<ModelMsg>,
    exit: bool,
    board: Option<Board>,
    cursor: CBPosition,
    square_selected: Option<CBPosition>,
    valid_moves: Option<Vec<CBPosition>>,
    game_data: GameData,
}

pub struct GameData {
    pub which_turn: Side,
    pub turn_count: usize,
    pub move_history: Vec<String>,
}
impl GameData {
    fn new() -> GameData {
        GameData {
            which_turn: Side::White,
            turn_count: 0,
            move_history: Vec::new(),
        }
    }
}

struct MoveRecord<'a> {
    moving_piece: &'a Piece,
    from_pos: CBPosition,
    destination: CBPosition,
    captures: bool,
    promotes_to: Option<PieceType>,
}
impl EdenChessUi {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        let _ = self.send.send(UiMsg::GetBoardState);
        while !self.exit {
            let _ = self.send.send(UiMsg::GetBoardState);
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_model_events();
            self.handle_events()?;
        }
        info!("Exiting");
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        if let Some(b) = &self.board {
            let block = Block::new().padding(Padding::symmetric(
                (frame.size().width as f32 * 0.05).floor() as u16,
                (frame.size().height as f32 * 0.05).floor() as u16,
            ));
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(block.inner(frame.size()));
            let left_panel = layout[0];
            let right_panel = layout[1];
            let valid_moves = if let Some(p) = self.square_selected {
                if let Some(ms) = &self.valid_moves {
                    ms
                } else {
                    &Vec::<CBPosition>::new()
                }
            } else {
                &Vec::<CBPosition>::new()
            };
            let ui_board = Chessboard::new(&b, self.cursor, valid_moves);
            frame.render_widget(ui_board, left_panel);
            frame.render_widget(Infobox::new(&self.game_data), right_panel)
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::poll(Duration::from_secs(0))? {
            true => match event::read() {
                Ok(e) => match e {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        self.handle_key_event(key_event)
                    }
                    _ => {}
                },
                Err(e) => error!("{}", e),
            },
            false => {} // it's important to check that the event is a key press event as
                        // crossterm also emits key release and repeat events on Windows.
        };
        Ok(())
    }
    //

    fn handle_key_event(&mut self, e: KeyEvent) {
        match e.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Left => self.cursor.move_cursor_left(),
            KeyCode::Right => self.cursor.move_cursor_right(),
            KeyCode::Up => self.cursor.move_cursor_up(),
            KeyCode::Down => self.cursor.move_cursor_down(),
            KeyCode::Char(' ') => {
                self.handle_space_pressed();
            }
            KeyCode::Esc => {
                self.square_selected = None;
                self.reset_valid_positions();
            }

            _ => {}
        };
    }

    fn handle_space_pressed(&mut self) {
        if let Some(selected_pos) = self.square_selected {
            if let Some(valids) = &self.valid_moves {
                if selected_pos != self.cursor && valids.contains(&self.cursor) {
                    if let Err(e) = self.send.send(UiMsg::MakeMove((selected_pos, self.cursor))) {
                        error!("{}", e)
                    } else {
                        self.game_data.turn_count += 1;
                        self.game_data.which_turn.flip();
                    };
                }
                self.square_selected = None;
            }
        } else {
            if let Some(board) = &self.board {
                if let Some(piece_under_cursor) = board
                    .get(&self.cursor)
                    .expect("Cursor should be within bounds")
                {
                    if piece_under_cursor.side == self.game_data.which_turn {
                        self.square_selected = Some(self.cursor);
                        let _ = self.send.send(UiMsg::GetValidMoves(self.cursor));
                    }
                }
            }
        }
        self.reset_valid_positions();
    }

    fn add_move_to_move_history(&mut self, selected: CBPosition) {
        if let Some(board) = &self.board {
            if let Some(piece) = board.get(&selected).expect("Selected should be in board") {
                let move_str = String::new();
            } else {
                error!("Selected place isnt occupied!")
            }
        };
    }

    fn handle_model_events(&mut self) {
        match self.recv.try_recv() {
            Ok(msg) => match msg {
                ModelMsg::Debug(d) => debug!("{}", d),
                ModelMsg::MoveIsInvalid => (),
                ModelMsg::Moves(ms) => self.valid_moves = Some(ms),
                ModelMsg::BoardState(b) => self.board = Some(b),
                ModelMsg::Stalemate => todo!(),
                ModelMsg::Checkmate(_) => todo!(),
            },
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => {}
                std::sync::mpsc::TryRecvError::Disconnected => error!("{}", e),
            },
        }
    }
    fn reset_valid_positions(&mut self) {
        self.valid_moves = None;
    }
}
