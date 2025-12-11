use std::{
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

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
pub enum GameState {
    Valid,
    Unextracted,
    Invalid(String),
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameState::Valid => "Valid".to_string(),
                GameState::Unextracted => "Unextracted".to_string(),
                GameState::Invalid(e) => format!("Invalid: {}", e),
            }
        )
    }
}

#[derive(Debug)]
pub struct GameManager {
    focused: bool,
    data_folder: PathBuf,
    iso_name: PathBuf,
    list_state: ListState,
    game_state: GameState,

    extracting: Arc<Mutex<bool>>,
}

impl Default for GameManager {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select_first();

        let mut game_manager = Self {
            focused: false,
            data_folder: "./data".into(),
            iso_name: "game.iso".into(),
            list_state,
            game_state: GameState::Unextracted,
            extracting: Arc::new(false.into()),
        };

        game_manager.refresh_game().unwrap_or_default();
        game_manager
    }
}

#[derive(Debug)]
enum GameManagerAction {
    ExtractGame,
    Refresh,
    ClearTempFiles,
}

impl Display for GameManagerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GameManagerAction::ExtractGame => "Extract the game",
                GameManagerAction::Refresh => "Refresh",
                GameManagerAction::ClearTempFiles => "Clear temporary files",
            }
        )
    }
}

const ACTIONS: [GameManagerAction; 3] = [
    GameManagerAction::Refresh,
    GameManagerAction::ExtractGame,
    GameManagerAction::ClearTempFiles,
];

impl GameManager {
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        if *self.extracting.lock().unwrap() {
            Paragraph::new("Please wait for the file extraction to complete.").render(area, buf);

            return;
        }

        let [options_area, data_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(5)]).areas(area);

        let output = format!(
            r"
Game iso location: {iso_location}
Game state: {game_state}

{details}
",
            iso_location = self.data_folder.join(&self.iso_name).display(),
            game_state = self.game_state,
            details = match &self.game_state {
                GameState::Valid =>
                    "The game files have been validated, and a randomiser can be created from these files.".to_string(),
                GameState::Unextracted =>
                    "The game files have not been extracted yet. Select \"Extract Game files\" to extract them.".to_string(),
                GameState::Invalid(e) => format!(
                    "The game files are in an invalid state. Error: {}\n\nYou can wipe the temporary files by selecting the \"Clear temporary files\" option.", e.to_string()),
            }
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

    pub fn extract_game(&mut self) {
        *self.extracting.lock().unwrap() = true;

        let extract_xiso_path: PathBuf = "extract-xiso".into();
        let iso_path = self
            .data_folder
            .canonicalize()
            .unwrap()
            .join(&self.iso_name);
        let extraction_path = self.data_folder.join("game");

        if !iso_path.is_file() {
            eprintln!("File {} does not exist.", iso_path.display());
        }

        match xbpatch_core::iso_handling::extract_iso(
            &extract_xiso_path,
            &iso_path,
            &extraction_path,
        ) {
            Ok(_) => {}
            Err(_error) => {
                // TODO: Log the error or display it somehow
            }
        };

        *self.extracting.lock().unwrap() = false;
        self.refresh_game().unwrap_or_default();
    }

    pub fn refresh_game(&mut self) -> Result<(), String> {
        if !self.data_folder.exists() {
            fs::create_dir_all(&self.data_folder).map_err(|e| e.to_string())?;
        }

        let game_files_path: PathBuf = self.data_folder.join("game");
        if !game_files_path.is_dir() {
            self.game_state = GameState::Unextracted;
            return Ok(());
        }

        let default_xbe_path = game_files_path.join("default.xbe");
        if !default_xbe_path.is_file() {
            self.game_state = GameState::Invalid(format!(
                "File \"default.xbe\" could not be found at {}",
                default_xbe_path.display()
            ))
        }

        let bundles_path = game_files_path.join("bundles");
        if !bundles_path.is_dir() {
            self.game_state = GameState::Invalid(format!(
                "File \"default.xbe\" could not be found at {}",
                default_xbe_path.display()
            ));

            return Ok(());
        }

        self.game_state = GameState::Valid;

        Ok(())
    }

    pub fn clear_temp_files(&mut self) -> Result<(), io::Error> {
        fs::remove_dir_all(self.data_folder.join("game"))?;

        self.refresh_game().unwrap_or_default();

        Ok(())
    }

    pub fn trigger(&mut self) {
        let Some(index) = self.list_state.selected() else {
            return;
        };

        if index >= ACTIONS.len() {
            return;
        }

        match ACTIONS[index] {
            GameManagerAction::ExtractGame => {
                self.extract_game();
            }
            GameManagerAction::Refresh => {
                self.refresh_game().unwrap_or_default();
            }
            GameManagerAction::ClearTempFiles => {
                self.clear_temp_files().unwrap_or_default();
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.list_state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.list_state.select_previous(),
            KeyCode::Enter => self.trigger(),
            _ => (),
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn extracting(&self) -> bool {
        *self.extracting.lock().unwrap()
    }

    pub fn data_folder(&self) -> &PathBuf {
        &self.data_folder
    }
}
