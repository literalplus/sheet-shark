use chrono::NaiveTime;
use crossterm::event::{KeyCode, KeyEvent};
use enum_dispatch::enum_dispatch;
use ratatui::{
    style::{Modifier, Style},
    text::Text,
    widgets::{Row, Table},
};

use crate::components::home::{
    action::HomeAction,
    state::{HomeState, TimeItem},
};

#[enum_dispatch]
pub trait EditModeBehaviour {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction;
    fn style_table<'a>(&self, table: Table<'a>) -> Table<'a>;
    fn style_selected_item<'a>(&self, item: &'a TimeItem) -> Row<'a> {
        item.as_row()
    }
}

#[derive(Default)]
pub struct Select {}

impl EditModeBehaviour for Select {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Up | KeyCode::Down => {
                if key.code == KeyCode::Up {
                    state.table.select_previous()
                } else {
                    state.table.select_next()
                }
                if state.table.selected_column().is_none() {
                    state.table.select_first_column();
                }
            }
            KeyCode::Left => state.table.select_previous_column(),
            KeyCode::Right => state.table.select_next_column(),
            KeyCode::Esc => state.table.select(None),
            KeyCode::Char(' ') => return HomeAction::EnterEdit,
            _ => {}
        }
        HomeAction::None
    }

    fn style_table<'a>(&self, table: Table<'a>) -> Table<'a> {
        table.cell_highlight_style(Style::from(Modifier::BOLD))
    }
}

#[derive(Default)]
pub struct Time {
    buf: String,
}

impl Time {
    fn new(state: &HomeState) -> Self {
        let item = state.expect_selected_item();
        Self {
            buf: item.start_time.format("%H%M").to_string(),
        }
    }
}

impl EditModeBehaviour for Time {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Esc => return HomeAction::ExitEdit,
            KeyCode::Enter => {
                let parsed = match NaiveTime::parse_from_str(&self.buf, "%H%M") {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        return HomeAction::SetStatusLine(format!("Invalid: {err}"));
                    }
                };
                state.expect_selected_item_mut().start_time = parsed;
                return HomeAction::ExitEdit;
            }
            KeyCode::Char(chr) => {
                if self.buf.len() < 4 {
                    self.buf.push(chr);
                }
            }
            KeyCode::Backspace => {
                if !self.buf.is_empty() {
                    self.buf.remove(self.buf.len() - 1);
                }
            }
            _ => {}
        }
        HomeAction::None
    }

    fn style_selected_item<'a>(&self, item: &'a TimeItem) -> Row<'a> {
        let mut cells = item.as_cells().clone();
        cells[0] = Text::from(self.buf.to_owned());
        Row::new(cells)
    }

    fn style_table<'a>(&self, table: Table<'a>) -> Table<'a> {
        table.cell_highlight_style(Style::from(Modifier::UNDERLINED))
    }
}

#[enum_dispatch(EditModeBehaviour)]
pub enum EditMode {
    Select,
    Time,
}

impl Default for EditMode {
    fn default() -> Self {
        EditMode::from(Select::default())
    }
}

impl EditMode {
    pub fn from_column_num(idx: usize, state: &HomeState) -> Option<Self> {
        Some(match idx {
            0 => EditMode::from(Time::new(state)),
            _ => return None,
        })
    }
}
