use crossterm::event::KeyEvent;
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

pub struct Ticket {
    buf: BufEditBehavior,
}

impl Ticket {
    pub fn new(state: &HomeState) -> Self {
        let item = state.expect_selected_item();
        Self {
            buf: BufEditBehavior::new(item.ticket.to_owned()),
        }
    }
}

impl EditModeBehavior for Ticket {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        if self.buf.should_save(key) {
            let ticket = if let Some(suggested) = state.tickets_suggestion.selected() {
                suggested.to_owned()
            } else {
                self.buf.to_owned()
            };
            state.expect_selected_item_mut().ticket = ticket;
            state.tickets_suggestion.reset();
        }
        let action = self.buf.handle_key_event(state, key);

        let suggest_query = &mut state.tickets_suggestion.query;
        if &self.buf != suggest_query {
            *suggest_query = self.buf.to_string();
            HomeAction::Many(vec![action, HomeAction::SuggestTickets(self.buf.to_string())])
        } else {
            action
        }
    }

    fn style_selected_item<'a>(&self, item: &'a TimeItem) -> Row<'a> {
        let mut cells = item.as_cells().clone();
        cells[2] = Text::from(self.buf.to_owned());
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
