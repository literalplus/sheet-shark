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

pub struct Description {
    buf: BufEditBehavior,
}

impl Description {
    pub fn new(state: &HomeState) -> Self {
        let item = state.expect_selected_item();
        Self {
            buf: BufEditBehavior::new(item.description.to_owned()),
        }
    }
}

impl EditModeBehavior for Description {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Enter => {
                state.expect_selected_item_mut().description = self.buf.to_owned();
                HomeAction::ExitEdit
            }
            _ => self.buf.handle_key_event(key),
        }
    }

    fn style_selected_item<'a>(&self, item: &'a TimeItem) -> Row<'a> {
        let mut cells = item.as_cells().clone();
        cells[2] = Text::from(self.buf.to_owned());
        Row::new(cells)
    }

    fn style_table<'a>(&self, table: Table<'a>) -> Table<'a> {
        table.cell_highlight_style(
            Style::from(Modifier::ITALIC)
                .not_reversed()
                .bg(tailwind::INDIGO.c300),
        )
    }
}
