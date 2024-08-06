use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

use super::chessboard::SquareColour;
use super::GameData;

pub struct Infobox<'a> {
    game_data: &'a GameData,
}

impl<'a> Infobox<'a> {
    pub fn new(game_data: &GameData) -> Infobox {
        Infobox { game_data }
    }

    fn create_layout(area: Rect) -> (Rect, Rect, Rect) {
        let temp_layout = Layout::default()
            .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
            .split(area);
        let (top, move_history_rect) = (temp_layout[0], temp_layout[1]);
        let temp_layout = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Horizontal)
            .split(top);
        let (turn_count_rect, turn_side_rect) = (temp_layout[0], temp_layout[1]);
        (turn_count_rect, turn_side_rect, move_history_rect)
    }
}
impl<'a> Widget for Infobox<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (turn_count_rect, turn_side_rect, move_history_rect) = Infobox::create_layout(area);

        let turn_count_block = Block::new().title("Turn Count");
        let turn_count_inner = turn_count_block.inner(turn_count_rect);
        turn_count_block.render(turn_count_rect, buf);
        let turn_count_para = Paragraph::new(self.game_data.turn_count.to_string());
        turn_count_para.render(turn_count_inner, buf);

        let turn_side_block = Block::new().title("Who's Turn").borders(Borders::ALL);
        let turn_side_inner = turn_side_block.inner(turn_side_rect);
        turn_side_block.render(turn_side_rect, buf);
        let turn_side_para = Paragraph::new(self.game_data.which_turn)
            .bg(SquareColour::from(self.game_data.which_turn)
                .to_color()
                .into())
            .fg(Color::Black)
            .centered();
        turn_side_para.render(turn_side_inner, buf);

        let move_history_str = self
            .game_data
            .move_history
            .iter()
            .fold("".to_owned(), |acc, m| format!("{}\n{}", acc, m));
        let move_history_block = Block::new().title("Move History");
        let move_history_inner = move_history_block.inner(move_history_rect);
        move_history_block.render(move_history_rect, buf);
        let move_history_para = Paragraph::new(move_history_str);
        move_history_para.render(move_history_inner, buf);
    }
}
