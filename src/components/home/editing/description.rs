use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Modifier, Style, Stylize, palette::tailwind},
    text::Text,
    widgets::{Row, Table},
};

use super::EditModeBehavior;
use crate::components::home::{
    action::HomeAction,
    editing::shared::BufEditBehavior,
    state::{HomeState, TimeItem},
};

pub struct Description {
    buf: BufEditBehavior,
}

impl Description {
    pub fn new(state: &HomeState) -> Self {
        let item = state.expect_selected_item();
        Self {
            buf: item.description.to_owned().into(),
        }
    }
}

impl EditModeBehavior for Description {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        if self.buf.should_save(key) {
            state.expect_selected_item_mut().description = self.buf.to_owned();
        }
        if key.code == KeyCode::Right {
            // UX feature: Since duration of this entry and time of the next entry represent the same information,
            // we skip the duration. It's usually more ergonomic to enter the time explicitly. If the user wants
            // to enter a duration instead, they can move left again. That use-case is also why this feature is
            // NOT implemented in the opposite direction.
            let in_last_row = state.is_last_row_selected();
            if in_last_row {
                let new_item = TimeItem::new(
                    Default::default(),
                    state.expect_selected_item().next_start_time(),
                );
                state.items.push(new_item);
            }
            state.table.select_next();
            state.table.select_column(Some(0));
            return HomeAction::None;
        }
        self.buf.handle_key_event(state, key)
    }

    fn style_selected_item<'a>(&self, item: &'a TimeItem) -> Row<'a> {
        let mut cells = item.as_cells().clone();
        cells[3] = Text::from(self.buf.to_owned());
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
