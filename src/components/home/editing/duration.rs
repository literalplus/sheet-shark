use crossterm::event::{KeyCode, KeyEvent};
use humantime::parse_duration;
use ratatui::{
    style::{Modifier, Style},
    text::Text,
    widgets::{Row, Table},
};

use super::{BufEditBehavior, EditModeBehavior};
use crate::components::home::{
    action::HomeAction,
    state::{HomeState, TimeItem},
};

#[derive(Default)]
pub struct Duration {
    buf: BufEditBehavior,
}

impl EditModeBehavior for Duration {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Enter => {
                if self.buf.parse::<u16>().is_ok() {
                    self.buf.push('m');
                }
                let parsed = match parse_duration(&self.buf) {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        return HomeAction::SetStatusLine(format!("Invalid: {err}"));
                    }
                };
                if parsed.as_secs() == 0 || parsed.as_secs() % 60 != 0 {
                    return HomeAction::SetStatusLine(
                        "Duration must be a whole number of minutes (e.g. 15m)".to_string(),
                    );
                }
                state.expect_selected_item_mut().duration = parsed;

                let my_item = state.expect_selected_item();
                let initial_last_item = state.items.len() - 1;
                if state.table.selected() == Some(initial_last_item) {
                    let new_item = TimeItem {
                        start_time: my_item.start_time + parsed,
                        ..TimeItem::default()
                    };
                    state.items.push(new_item);
                    state.table.select_last();
                    state.table.select_first_column();
                }

                HomeAction::ExitEdit
            }
            _ => self.buf.handle_key_event(key),
        }
    }

    fn style_selected_item<'a>(&self, item: &'a TimeItem) -> Row<'a> {
        let mut cells = item.as_cells().clone();
        let mut content = self.buf.to_owned();
        if content.is_empty() {
            content = "...".into();
        }
        cells[3] = Text::from(content);
        Row::new(cells)
    }

    fn style_table<'a>(&self, table: Table<'a>) -> Table<'a> {
        table.cell_highlight_style(Style::from(Modifier::ITALIC))
    }
}
