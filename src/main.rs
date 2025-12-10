mod styles;

use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{
        Alignment::{self},
        Constraint, Layout, Rect,
    },
    style::Stylize,
    widgets::{Block, Paragraph, Widget},
};

use color_eyre::Result;

use crate::{
    main_list::{MainOption, MainOptionsList},
    randomiser::RandomiserState,
};

mod main_list;

mod randomiser;

#[derive(Debug)]
struct App {
    should_exit: bool,
    main_options_list: MainOptionsList,
    randomiser_state: RandomiserState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            randomiser_state: Default::default(),
            main_options_list: Default::default(),
        }
    }
}

impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        let [title_area, version_area] =
            Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(area);

        Paragraph::new("Ghoulies Randomiser")
            .bold()
            .left_aligned()
            .render(title_area, buf);

        Paragraph::new(format!("v{}", env!("CARGO_PKG_VERSION")))
            .bold()
            .right_aligned()
            .render(version_area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        const TITLE_LABEL: &str = "Controls";

        let [title_area, content_area] = Layout::horizontal([
            Constraint::Length((TITLE_LABEL.len() + 2) as u16),
            Constraint::Fill(1),
        ])
        .areas(area);

        Paragraph::new(TITLE_LABEL).render(title_area, buf);

        const CONTROLS: [&str; 3] = ["←↓↑→/hjkl - Move", "Enter - Select", "q/Esc - Go Back"];

        let block = Block::new()
            .title_alignment(Alignment::Right)
            .title(CONTROLS.join("    "));

        block.render(content_area, buf);
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let result = App::default().run(terminal);
    ratatui::restore();
    result
}

enum MoveDirection {
    Left,
    Right,
    Up,
    Down,
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        // Select the first option on startup
        self.main_options_list.list_state.select_first();
        self.randomiser_state.list_state.select_first();

        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    fn render_main_item(&mut self, area: Rect, buf: &mut Buffer) {
        match self.main_options_list.current_hovered_status() {
            MainOption::CreateRandomiser => self.randomiser_state.render(area, buf),
            MainOption::BuildAssetLibrary => {
                Paragraph::new("Build asset library coming soon").render(area, buf);
            }
        }
    }

    fn move_in_direction(&mut self, direction: MoveDirection) {
        if self.main_options_list.focused() {
            match direction {
                MoveDirection::Down => self.main_options_list.list_state.select_next(),
                MoveDirection::Up => self.main_options_list.list_state.select_previous(),
                _ => (),
            }
        } else if self.randomiser_state.focused() {
            match direction {
                MoveDirection::Down => self.randomiser_state.list_state.select_next(),
                MoveDirection::Up => self.randomiser_state.list_state.select_previous(),
                _ => (),
            }
        }
    }

    fn go_back(&mut self) {
        if self.main_options_list.focused() {
            self.should_exit = true;
        } else if self.randomiser_state.focused() {
            self.randomiser_state.set_focused(false);
            self.main_options_list.set_focused(true);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.go_back(),
            // KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            // KeyCode::Char('l') | KeyCode::Right => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.move_in_direction(MoveDirection::Down),
            KeyCode::Char('k') | KeyCode::Up => self.move_in_direction(MoveDirection::Up),
            // KeyCode::Char('g') | KeyCode::Home => self.main_options_list.list_state.select_first(),
            // KeyCode::Char('G') | KeyCode::End => self.main_options_list.list_state.select_last(),
            KeyCode::Enter => {
                if self.main_options_list.focused() {
                    match self.main_options_list.current_hovered_status() {
                        MainOption::CreateRandomiser => {
                            self.randomiser_state.set_focused(true);
                        }
                        MainOption::BuildAssetLibrary => return,
                    }

                    self.main_options_list.set_focused(false);
                } else if self.randomiser_state.focused() {
                    self.randomiser_state.trigger();
                }
            }
            _ => {}
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(main_area);

        App::render_header(header_area, buf);

        self.main_options_list.render(list_area, buf);
        self.render_main_item(item_area, buf);
        self.main_options_list.render(list_area, buf);

        App::render_footer(footer_area, buf);
        // self.render_selected_item(item_area, buf);
    }
}
