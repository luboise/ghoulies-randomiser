use ratatui::style::{
    Color, Modifier, Style,
    palette::{material::BLUE, tailwind::SLATE},
};

pub const HOVERING_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
pub const FOCUSED_STYLE: Style = HOVERING_STYLE.add_modifier(Modifier::RAPID_BLINK);
pub const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
pub const NORMAL_ROW_BG: Color = SLATE.c950;
