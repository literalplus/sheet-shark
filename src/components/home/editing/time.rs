use chrono::NaiveTime;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Modifier, Style, Stylize, palette::tailwind},
    text::Text,
    widgets::{Row, Table},
};

use super::{BufEditBehavior, EditModeBehavior};
use crate::components::home::{
    action::HomeAction,
    state::{HomeState, TimeItem},
};

pub struct Time {
    buf: BufEditBehavior,
}

impl Time {
    pub fn new(state: &HomeState) -> Self {
        let item = state.expect_selected_item();
        Self {
            buf: BufEditBehavior::new(item.start_time.format("%H%M")),
        }
    }
}

impl EditModeBehavior for Time {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Enter => {
                let parsed = match NaiveTime::parse_from_str((&self.buf).into(), "%H%M") {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        return HomeAction::SetStatusLine(format!("Invalid: {err}"));
                    }
                };
                state.expect_selected_item_mut().start_time = parsed;
                HomeAction::ExitEdit
            }
            KeyCode::Char(_) if self.buf.len() >= 4 => HomeAction::None,
            _ => self.buf.handle_key_event(key),
        }
    }

    fn style_selected_item<'a>(&self, item: &'a TimeItem) -> Row<'a> {
        let mut cells = item.as_cells().clone();
        cells[0] = Text::from(self.buf.to_owned());
        Row::new(cells)
    }

    fn style_table<'a>(&self, table: Table<'a>) -> Table<'a> {
        table.cell_highlight_style(
            Style::from(Modifier::UNDERLINED)
                .not_reversed()
                .bg(tailwind::INDIGO.c300),
        )
    }
}
