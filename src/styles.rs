use ratatui::style::{Modifier, Style, palette::tailwind::SLATE};

pub const HOVERING_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
pub const FOCUSED_STYLE: Style = HOVERING_STYLE.add_modifier(Modifier::RAPID_BLINK);
