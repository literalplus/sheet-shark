use chrono::NaiveTime;
use color_eyre::eyre::{Result, eyre};
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

    fn handle_save(&self, state: &mut HomeState) -> Result<()> {
        let parsed = NaiveTime::parse_from_str((&self.buf).into(), "%H%M");
        let parsed = parsed.map_err(|err| eyre!("invalid: {err}"))?;
        state.expect_selected_item_mut().start_time = parsed;
        Ok(())
    }
}

impl EditModeBehavior for Time {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        if self.buf.should_save(key)
            && let Err(err) = self.handle_save(state)
        {
            return err.into();
        }
        match key.code {
            KeyCode::Enter => HomeAction::ExitEdit,
            KeyCode::Char(_) if self.buf.len() >= 4 => HomeAction::None,
            _ => self.buf.handle_key_event(state, key),
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
