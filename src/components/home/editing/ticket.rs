use std::ops::Deref;

use crossterm::event::{KeyCode, KeyEvent};
use itertools::Itertools;
use ratatui::{
    layout::Constraint,
    style::{Modifier, Style, Stylize, palette::tailwind},
    text::{Line, Text},
    widgets::{ListItem, ListState, Row, Table, TableState},
};

use super::EditModeBehavior;
use crate::{
    components::home::{
        action::HomeAction,
        editing::shared::BufEditBehavior,
        state::{HomeState, TimeItem},
    },
    persist::Event,
    widgets::table_popup::TablePopup,
};

pub struct Ticket {
    buf: BufEditBehavior,
    suggestion: TicketsSuggestion,
}

impl Ticket {
    pub fn new(state: &HomeState) -> Self {
        let item = state.expect_selected_item();
        Self {
            buf: item.ticket.to_owned().into(),
            suggestion: Default::default(),
        }
    }
}

impl EditModeBehavior for Ticket {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        match self.suggestion.handle_key_event(key) {
            SuggestAction::Done => return HomeAction::None,
            SuggestAction::Accept(suggested) => {
                self.buf = suggested.into();
            }
            SuggestAction::None => {}
        }

        if self.buf.should_save(key) {
            state.expect_selected_item_mut().ticket = self.buf.to_owned();
        }

        let action = self.buf.handle_key_event(state, key);

        if self.buf != self.suggestion.query {
            self.suggestion.query = self.buf.to_string();
            action + HomeAction::SuggestTickets(self.buf.to_string())
        } else {
            action
        }
    }

    fn style_selected_item<'a>(&self, item: &'a TimeItem) -> Row<'a> {
        let mut cells = item.as_cells(false).clone();
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

    fn draw_popup<'a, CI>(
        &'a mut self,
        table_state: &'a TableState,
        constraints: CI,
    ) -> Option<TablePopup<'a>>
    where
        CI: IntoIterator<Item = Constraint>,
    {
        if self.suggestion.is_active() {
            Some(self.suggestion.as_popup(table_state, constraints))
        } else {
            None
        }
    }

    fn handle_persisted(&mut self, event: Event) {
        if let Event::TicketsSuggested { query, ticket_keys } = event {
            self.suggestion.handle_result(query, ticket_keys);
        }
    }
}

#[derive(Default)]
struct TicketsSuggestion {
    query: String,
    suggestions: Vec<String>,
    list_state: ListState,
}

enum SuggestAction {
    None,
    Done,
    Accept(String),
}

impl TicketsSuggestion {
    pub fn is_active(&self) -> bool {
        !self.query.is_empty() && !self.suggestions.is_empty()
    }

    pub fn handle_result(&mut self, query: String, suggestions: Vec<String>) {
        if query != self.query {
            return; // outdated result, new query in flight
        }
        let no_suggestions_before = self.suggestions.is_empty();
        self.suggestions = suggestions;
        if no_suggestions_before && !self.suggestions.is_empty() {
            self.list_state.select_first();
        }
    }

    pub fn selected(&self) -> Option<&str> {
        if let Some(idx) = self.list_state.selected() {
            self.suggestions.get(idx).map(|x| x.as_str())
        } else {
            None
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> SuggestAction {
        if !self.is_active() {
            return SuggestAction::None;
        }
        match key.code {
            KeyCode::Down => {
                self.list_state.select_next();
                SuggestAction::Done
            }
            KeyCode::Up => {
                if Some(0) == self.list_state.selected() {
                    self.list_state.select(None);
                } else {
                    self.list_state.select_previous();
                }
                SuggestAction::Done
            }
            KeyCode::Esc => {
                *self = Default::default();
                SuggestAction::Done
            }
            KeyCode::Tab | KeyCode::Enter | KeyCode::Right => {
                if let Some(suggested) = self.selected() {
                    let suggested = suggested.to_owned();
                    *self = Default::default();
                    SuggestAction::Accept(suggested)
                } else {
                    SuggestAction::None
                }
            }
            _ => SuggestAction::None,
        }
    }

    pub fn as_popup<'a, CI>(
        &'a mut self,
        table_state: &'a TableState,
        constraints: CI,
    ) -> TablePopup<'a>
    where
        CI: IntoIterator<Item = Constraint>,
    {
        let items = self
            .suggestions
            .iter()
            .map(|it| ListItem::from(Line::from(it.deref())))
            .collect_vec();
        let state = &mut self.list_state;
        TablePopup::new(table_state, state, items, constraints)
    }
}
