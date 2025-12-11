use std::{env, fs, path::Path};

use bnl::{
    BNLError, BNLFile,
    asset::{Asset, aidlist::AidList},
    modding::Mod,
};
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{
        Color, Style, Stylize,
        palette::{material::BLUE, tailwind::SLATE},
    },
    symbols,
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, StatefulWidget},
};
use regex::Regex;

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
    // children: Vec<RandomiserOption>,
}

impl RandomiserOption {
    pub fn as_list_item(&self, depth: usize) -> ListItem {
        let text = match self.value {
            RandomiserOptionValue::Bool(bool) => {
                format!("{} {}", if bool { "☑" } else { "☐" }, self.name)
            }
            RandomiserOptionValue::Float(f) => format!("{}: {f:.2}", self.name),
            RandomiserOptionValue::Uint64(u) => format!("{}: {u}", self.name),
        };

        ListItem::new(format!("{:>width$} {}", "", text, width = depth * 4))
    }

    pub fn as_list_item_tree(&self, depth: usize) -> Vec<ListItem> {
        let items = vec![self.as_list_item(depth)];

        /*
        self.children.iter().for_each(|child| {
            items.extend(child.as_list_item_tree(depth + 1));
        });
        */

        items
    }
}

impl RandomiserOption {
    pub fn default_randomiser_options() -> Vec<RandomiserOption> {
        vec![
            RandomiserOption {
                name: "Randomise Room Order".to_string(),
                value: RandomiserOptionValue::Bool(false),
                // children: vec![],
            },
            RandomiserOption {
                name: "Remove \"Book Cutscenes\"".to_string(),
                value: RandomiserOptionValue::Bool(false),
                // children: vec![],
            },
            RandomiserOption {
                name: "Seed".to_string(),
                value: RandomiserOptionValue::Uint64(rand::random()),
                /*
                children: vec![RandomiserOption {
                    name: "SEED CHILD???".to_string(),
                    value: RandomiserOptionValue::Bool(false),
                    children: vec![],
                }],
                */
            },
        ]
    }
}

#[derive(Debug)]
pub struct RandomiserState {
    root_options: Vec<RandomiserOption>,
    focused: bool,
    pub list_state: ListState,
}

/*
pub struct RandomiserOptionIterator<'a> {
    root_option: &'a RandomiserOption,
    child_path: Vec<usize>,
}

impl Iterator for RandomiserOptionIterator {
    type Item = RandomiserOption;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current_option = self.root_option;

        for child_index in &self.child_path {
            current_option = current_option.children[child_index];
        }
    }
}
*/

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

    pub fn options(&self) -> &[RandomiserOption] {
        &self.root_options
    }

    pub fn get_option(&self, index: usize) -> Option<&RandomiserOption> {
        self.root_options.get(index)
    }

    pub fn options_mut(&mut self) -> &mut [RandomiserOption] {
        &mut self.root_options
    }

    pub fn get_option_mut(&mut self, index: usize) -> Option<&mut RandomiserOption> {
        self.root_options.get_mut(index)
    }

    pub fn focused(&self) -> bool {
        self.focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn trigger(&mut self) {
        if !self.focused {
            // eprintln!(
            //     "Error: MainOptionsList triggered when it was not focused. Ignoring this and continuing."
            // );
            return;
        }

        let Some(index) = self.list_state.selected() else {
            eprintln!("Error: No value selected in MainOptionsList. Unable to trigger.");
            return;
        };

        // If we selected the last option
        if index == self.root_options.len() {
            randomise_game(self, env::args().collect::<Vec<String>>().get(1).unwrap())
                .expect("Failed to randomise game.");
            return;
        }

        let Some(option) = self.root_options.get_mut(index) else {
            return;
        };

        match &mut option.value {
            RandomiserOptionValue::Bool(b) => *b = !*b,
            RandomiserOptionValue::Float(f) => *f = rand::random(),
            RandomiserOptionValue::Uint64(u) => *u = rand::random(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.list_state.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.list_state.select_previous(),
            KeyCode::Backspace => {
                let Some(option) = self.get_current_randomiser_option_mut() else {
                    return;
                };

                match &mut option.value {
                    RandomiserOptionValue::Float(_) => (),
                    RandomiserOptionValue::Bool(v) => *v = !*v,
                    RandomiserOptionValue::Uint64(v) => *v = v.saturating_div(10),
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let num: u64 = c.to_digit(10).unwrap_or(0).into();

                let Some(option) = self.get_current_randomiser_option_mut() else {
                    return;
                };

                match &mut option.value {
                    RandomiserOptionValue::Uint64(v) => {
                        *v = v
                            .checked_mul(10)
                            .unwrap_or(*v)
                            .checked_add(num)
                            .unwrap_or(*v)
                    }
                    RandomiserOptionValue::Bool(_) => (),
                    RandomiserOptionValue::Float(_) => todo!(),
                }
            }

            _ => (),
        }
    }

    pub fn get_current_randomiser_option(&self) -> Option<&RandomiserOption> {
        let Some(index) = self.list_state.selected() else {
            return None;
        };

        self.root_options.get(index)
    }

    pub fn get_current_randomiser_option_mut(&mut self) -> Option<&mut RandomiserOption> {
        let Some(index) = self.list_state.selected() else {
            return None;
        };

        self.root_options.get_mut(index)
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

        let mut items: Vec<ListItem> = {
            self.root_options
                .iter()
                .flat_map(|root_option| root_option.as_list_item_tree(0))
                .collect()
        };

        items.push(ListItem::new("Create."));

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

pub fn randomise_game<P: AsRef<Path>>(
    state: &RandomiserState,
    common_bnl_path: P,
) -> Result<(), BNLError> {
    let mut bnl = BNLFile::from_bytes(&fs::read(&common_bnl_path)?)?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

    let out_dir = "./out";

    let mut randomiser_mod = Mod::new("Randomiser");

    let mut rng = StdRng::seed_from_u64(match state.root_options.get(2).unwrap().value {
        RandomiserOptionValue::Uint64(seed) => seed,
        _ => 0,
    });

    let randomise_rooms =
        if let RandomiserOptionValue::Bool(b) = state.root_options.get(0).unwrap().value {
            b
        } else {
            false
        };

    let remove_book_cutscenes =
        if let RandomiserOptionValue::Bool(b) = state.root_options.get(1).unwrap().value {
            b
        } else {
            false
        };

    randomiser_mod.overrides_mut();

    let re = Regex::new(r"aid_script_ghoulies_.*scene[0-9]+_.*[book].*").unwrap();

    bnl.modify_asset(
        "aid_aidlist_ghoulies_sceneorder_game",
        |list: &mut Asset<AidList>| {
            if randomise_rooms {
                list.asset_mut().asset_ids_mut()[3..130].shuffle(&mut rng);
            }

            if remove_book_cutscenes {
                list.asset_mut()
                    .asset_ids_mut()
                    .retain(|aid| !re.is_match(aid));
            }

            Ok(())
        },
    )
    .expect("Failed to remove aidlist");

    fs::create_dir_all(out_dir).expect("Failed to create out dir.");

    fs::copy(
        &common_bnl_path,
        format!(
            "{out_dir}/{}_backup_{}",
            &common_bnl_path
                .as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            timestamp
        ),
    )
    .expect("Failed to backup bnl file.");

    fs::write(common_bnl_path, bnl.to_bytes()).expect("Failed to write new bnl.");

    Ok(())
}
