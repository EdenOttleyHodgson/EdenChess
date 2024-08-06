use std::iter::repeat;
use std::iter::zip;

use crate::control;
use crate::control::CBPosition;
use crate::model::Side;
use crate::model::{Board, Piece};
use color_eyre::owo_colors::Color as BadColor2;
use color_eyre::owo_colors::OwoColorize as BadColor;
use crossterm::style::Color;
use log::debug;
use log::info;
use log::warn;
use ratatui::prelude::*;
use ratatui::widgets as w;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

#[derive(Clone, Copy)]
pub struct Chessboard<'a> {
    board: &'a Board,
    cursor: CBPosition,
    valid_moves: &'a Vec<CBPosition>,
}
impl<'a> Chessboard<'a> {
    pub fn new(
        board: &'a Board,
        cursor: CBPosition,
        valid_moves: &'a Vec<CBPosition>,
    ) -> Chessboard<'a> {
        Chessboard {
            board,
            cursor,
            valid_moves,
        }
    }
}

impl<'a> Widget for Chessboard<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Paragraph::new("Plog").render(area, buf);

        let (board_size_width, board_size_height) = if true || area.width > area.height {
            ((area.height as f32 * 2.5).floor() as u16, area.height)
        } else {
            ((area.height as f32 * 2.5).floor() as u16, area.height)
            //(area.width, (area.width as f32 * 2.5).floor() as u16)
        };
        let square_width = (board_size_width as f32 / 9.0).floor() as u16;
        let square_height = (board_size_height as f32 / 9.0).floor() as u16;
        let mut col_constraints = Constraint::from_mins(repeat(square_height).take(9));
        col_constraints.append(&mut Constraint::from_maxes(repeat(square_height).take(9)));
        let mut row_constraints = Constraint::from_mins(repeat(square_width).take(9));
        row_constraints.append(&mut Constraint::from_maxes(repeat(square_height).take(9)));

        let board_rect = Rect::new(area.x, area.y, board_size_width, board_size_height);

        let whole_board = Layout::default()
            .direction(Direction::Vertical)
            .constraints(col_constraints)
            .split(board_rect);

        for row in (0..=8) {
            let mut current_colour = if row % 2 == 0 {
                SquareColour::White
            } else {
                SquareColour::Black
            };

            let row_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(row_constraints.clone())
                .split(whole_board[8 - row]);

            if row == 0 {
                for (col, col_let) in zip((1..=8).rev(), ('a'..='h').rev()) {
                    Paragraph::new(String::from(col_let)).render(row_layout[col], buf)
                }
            } else {
                Paragraph::new(row.to_string()).render(row_layout[0], buf);
                for (col, col_let) in zip((1..=8).rev(), ('a'..='h').rev()) {
                    let piece = match self.board.get(&CBPosition { col: col_let, row }) {
                        Some(p) => p,
                        None => &None,
                    };
                    let pos = CBPosition { col: col_let, row };
                    let square = ChessboardSquare {
                        piece,
                        colour: current_colour,
                        selected: self.cursor == pos,
                        valid: self.valid_moves.contains(&pos),
                    };
                    square.render(row_layout[col], buf);
                    current_colour.flip();
                }
            }
        }
    }
}

struct ChessboardSquare<'a> {
    piece: &'a Option<Piece>,
    colour: SquareColour,
    selected: bool,
    valid: bool,
}
impl<'a> Widget for ChessboardSquare<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (square_text, fg_color) = match self.piece {
            Some(p) => match p.piece_type {
                crate::model::PieceType::King => ("K", p.side.to_color()),

                crate::model::PieceType::Queen => ("Q", p.side.to_color()),

                crate::model::PieceType::Rook => ("R", p.side.to_color()),

                crate::model::PieceType::Bishop => ("B", p.side.to_color()),

                crate::model::PieceType::Knight => ("N", p.side.to_color()),

                crate::model::PieceType::Pawn => ("P", p.side.to_color()),
            },
            None => ("", ratatui::prelude::Color::Black), //Block::new().bg(self.colour.to_color().into()),
        };
        let mut square_text = square_text.to_string();
        if self.selected {
            Paragraph::new(square_text)
                .fg(fg_color)
                .bg(self.colour.to_selected_color().into())
                .render(area, buf);
        } else if self.valid {
            Paragraph::new(square_text)
                .fg(fg_color)
                .bg(SQUARE_VALID_FOR_MOVE.into())
                .render(area, buf);
        } else {
            Paragraph::new(square_text)
                .fg(fg_color)
                .bg(self.colour.to_color().into())
                .render(area, buf);
        };
    }
}

#[derive(Clone, Copy)]
pub enum SquareColour {
    Black,
    White,
}

static BLACK_SQUARE_COLOUR: Color = Color::Rgb {
    r: 76,
    g: 58,
    b: 46,
};

static WHITE_SQUARE_COLOUR: Color = Color::Rgb {
    r: 245,
    g: 226,
    b: 183,
};

static BLACK_SQUARE_COLOUR_SELECTED: Color = Color::Rgb {
    r: 115,
    g: 93,
    b: 78,
};

static WHITE_SQUARE_COLOUR_SELECTED: Color = Color::Rgb {
    r: 184,
    g: 120,
    b: 134,
};

static SQUARE_VALID_FOR_MOVE: Color = Color::Rgb {
    r: 50,
    g: 255,
    b: 50,
};

impl SquareColour {
    pub fn flip(&mut self) {
        match self {
            SquareColour::Black => *self = SquareColour::White,
            SquareColour::White => *self = SquareColour::Black,
        }
    }
    pub fn to_color(&self) -> Color {
        match self {
            SquareColour::Black => BLACK_SQUARE_COLOUR,
            SquareColour::White => WHITE_SQUARE_COLOUR,
        }
    }
    pub fn to_selected_color(&self) -> Color {
        match self {
            SquareColour::Black => BLACK_SQUARE_COLOUR_SELECTED,
            SquareColour::White => WHITE_SQUARE_COLOUR_SELECTED,
        }
    }
}
impl From<Side> for SquareColour {
    fn from(value: Side) -> Self {
        match value {
            Side::White => SquareColour::White,
            Side::Black => SquareColour::Black,
        }
    }
}
impl Into<Side> for SquareColour {
    fn into(self) -> Side {
        match self {
            SquareColour::Black => Side::Black,
            SquareColour::White => Side::White,
        }
    }
}
