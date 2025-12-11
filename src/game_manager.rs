use std::{fmt::Display, path::PathBuf};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget,
    },
};

use crate::styles::{FOCUSED_STYLE, NORMAL_ROW_BG, TODO_HEADER_STYLE};

#[derive(Debug)]
pub struct GameManager {
    focused: bool,
    data_folder: PathBuf,
    iso_name: PathBuf,
    list_state: ListState,
}

impl Default for GameManager {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select_first();

        Self {
            focused: false,
            data_folder: "./data".into(),
            iso_name: "game.iso".into(),
            list_state: list_state,
        }
    }
}

enum GameManagerAction {
    ExtractGame,
    Refresh,
}

impl Display for GameManagerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameManagerAction::ExtractGame => "Extract the game",
                GameManagerAction::Refresh => "Refresh",
            }
        )
    }
}

const ACTIONS: [GameManagerAction; 2] =
    [GameManagerAction::Refresh, GameManagerAction::ExtractGame];

impl GameManager {
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let [options_area, data_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(5)]).areas(area);

        let output = format!(
            "Game iso location: {iso_location}",
            iso_location = self.iso_name.display()
        );

        Paragraph::new(output)
            .bold()
            .left_aligned()
            .render(data_area, buf);

        let items: Vec<ListItem> = ACTIONS
            .map(|action| ListItem::new(action.to_string()))
            .to_vec();

        let block = Block::new()
            .title(Line::raw("Game Manager").centered())
            .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let list = if self.focused {
            List::new(items)
                .block(block)
                .highlight_style(FOCUSED_STYLE)
                .highlight_symbol(">")
                .highlight_spacing(HighlightSpacing::Always)
        } else {
            List::new(items)
                .block(block)
                .highlight_symbol(" ")
                .highlight_spacing(HighlightSpacing::Always)
        };

        StatefulWidget::render(list, options_area, buf, &mut self.list_state);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.list_state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.list_state.select_previous(),
            _ => (),
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn focused(&self) -> bool {
        self.focused
    }
}
