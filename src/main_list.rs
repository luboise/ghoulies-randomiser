use std::fmt::Display;

use color_eyre::owo_colors::OwoColorize;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{
        Color, Modifier, Style, Stylize,
        palette::tailwind::{BLUE, GREEN, SLATE},
    },
    symbols,
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, StatefulWidget},
};

use crate::styles::{FOCUSED_STYLE, HOVERING_STYLE};

#[derive(Debug, Clone)]
pub enum MainOption {
    CreateRandomiser,
    BuildAssetLibrary,
}

impl Display for MainOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MainOption::CreateRandomiser => "Create a randomiser",
                MainOption::BuildAssetLibrary => "Build asset library",
            }
        )
    }
}

#[derive(Debug)]
pub struct MainOptionsList {
    options: Vec<MainOption>,
    focused: bool,
    pub list_state: ListState,
}

impl Default for MainOptionsList {
    fn default() -> Self {
        Self {
            options: [MainOption::CreateRandomiser, MainOption::BuildAssetLibrary].to_vec(),
            list_state: Default::default(),
            focused: true,
        }
    }
}

impl MainOptionsList {
    pub fn current_hovered_status(&self) -> MainOption {
        self.list_state
            .selected()
            .and_then(|v| self.options.get(v).cloned())
            .unwrap_or(MainOption::CreateRandomiser)
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
        const NORMAL_ROW_BG: Color = SLATE.c950;

        let block = Block::new()
            .title(Line::raw("Options").centered())
            .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let items: Vec<ListItem> = self
            .options
            .iter()
            .map(|item| ListItem::from(item.to_string()))
            .collect();

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

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}
