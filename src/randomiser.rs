use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{
        Color, Style, Stylize,
        palette::{material::BLUE, tailwind::SLATE},
    },
    symbols,
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, StatefulWidget},
};

use crate::styles::FOCUSED_STYLE;

#[derive(Debug)]
pub enum RandomiserOptionValue {
    Float(f32),
    Bool(bool),
    Uint64(u64),
}

#[derive(Debug)]
pub struct RandomiserOption {
    name: String,
    value: RandomiserOptionValue,
    children: Vec<RandomiserOption>,
}

impl RandomiserOption {
    pub fn as_list_item(&self, depth: usize) -> ListItem {
        ListItem::new(format!(
            "{:>width$} {}",
            "",
            self.name.clone(),
            width = depth * 4
        ))
    }

    pub fn as_list_item_tree(&self, depth: usize) -> Vec<ListItem> {
        let mut items = vec![self.as_list_item(depth)];

        self.children.iter().for_each(|child| {
            items.extend(child.as_list_item_tree(depth + 1));
        });

        items
    }
}

impl RandomiserOption {
    pub fn default_randomiser_options() -> Vec<RandomiserOption> {
        vec![RandomiserOption {
            name: "Seed".to_string(),
            value: RandomiserOptionValue::Uint64(rand::random()),
            children: vec![RandomiserOption {
                name: "SEED CHILD???".to_string(),
                value: RandomiserOptionValue::Bool(false),
                children: vec![],
            }],
        }]
    }
}

#[derive(Debug)]
pub struct RandomiserState {
    root_options: Vec<RandomiserOption>,
    focused: bool,
    pub list_state: ListState,
}

impl Default for RandomiserState {
    fn default() -> Self {
        let list_state = ListState::default();

        Self {
            root_options: RandomiserOption::default_randomiser_options(),
            focused: false,
            list_state,
        }
    }
}

impl RandomiserState {
    pub fn get_list_items(&self) -> Vec<ListItem> {
        self.root_options
            .iter()
            .flat_map(|root_option| root_option.as_list_item_tree(0))
            .collect()
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
            .title(Line::raw("Configure Randomiser").centered())
            .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let items: Vec<ListItem> = {
            self.root_options
                .iter()
                .flat_map(|root_option| root_option.as_list_item_tree(0))
                .collect()
        };

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

        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}
