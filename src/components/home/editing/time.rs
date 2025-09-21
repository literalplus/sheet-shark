use std::time::Duration;

use chrono::{NaiveTime, TimeDelta};
use color_eyre::eyre::{Result, bail, eyre};
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

#[derive(Default)]
pub struct Time {
    buf: BufEditBehavior,
}

impl Time {
    pub fn new(state: &HomeState) -> Self {
        let item = state.expect_selected_item();
        Self {
            buf: item.start_time.format("%H%M").to_string().into(),
        }
    }

    fn handle_save(&self, state: &mut HomeState) -> Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }

        let parsed = NaiveTime::parse_from_str((&self.buf).into(), "%H%M");
        let parsed = parsed.map_err(|err| eyre!("invalid: {err}"))?;

        self.ensure_not_before_previous(state, parsed)?;
        self.ensure_not_after_next(state, parsed)?;

        // The idea for editing time is that it's taken away from the previous item and added to self
        self.adjust_previous_item(state, parsed);
        self.adjust_self(state, parsed);

        state.expect_selected_item_mut().start_time = parsed;
        Ok(())
    }

    fn ensure_not_before_previous(
        &self,
        state: &mut HomeState,
        my_next_time: NaiveTime,
    ) -> Result<()> {
        if state.table.selected() == Some(0) {
            return Ok(());
        }

        let previous_index = state.table.selected().unwrap_or(1) - 1;
        let previous_time = state
            .items
            .get(previous_index)
            .expect("previous item to exist")
            .start_time;

        if my_next_time < previous_time {
            bail!("cannot be before previous item's start ({previous_time})")
        }
        Ok(())
    }

    fn ensure_not_after_next(&self, state: &mut HomeState, my_next_time: NaiveTime) -> Result<()> {
        if state.is_last_row_selected() {
            return Ok(());
        }

        let next_index = state.table.selected().unwrap_or(0) + 1;
        let next_time = state
            .items
            .get(next_index)
            .expect("next item to exist")
            .start_time;

        if my_next_time > next_time {
            bail!("cannot be after next item's start ({next_time})")
        }
        Ok(())
    }

    fn adjust_previous_item(&self, state: &mut HomeState, my_next_time: NaiveTime) {
        if state.table.selected() == Some(0) {
            return;
        }

        let my_index = state.table.selected().expect("self selected for prev");
        let my_previous_time = state.expect_selected_item().start_time;

        let previous_item = state
            .items
            .get_mut(my_index - 1)
            .expect("previous item to exist");

        let time_delta = my_next_time - my_previous_time; // signed as opposed to Duration
        let duration_unsigned = Duration::from_secs(time_delta.num_seconds().unsigned_abs());
        if time_delta < TimeDelta::zero() {
            previous_item.duration -= duration_unsigned;
        } else {
            previous_item.duration += duration_unsigned;
        }
    }

    fn adjust_self(&self, state: &mut HomeState, my_next_time: NaiveTime) {
        let my_item = state.expect_selected_item_mut();
        let my_previous_time = my_item.start_time;
        if my_item.duration.is_zero() {
            return; // Last item doesn't have an end yet, don't hallucinate one
        }

        let time_delta = my_next_time - my_previous_time; // signed as opposed to Duration
        let duration_unsigned = Duration::from_secs(time_delta.num_seconds().unsigned_abs());
        if time_delta < TimeDelta::zero() {
            my_item.duration += duration_unsigned;
        } else {
            my_item.duration -= duration_unsigned;
        }
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
        let mut content = self.buf.to_owned();
        if content.is_empty() {
            content = format!("{}", item.start_time.format("%H%M"));
        }
        cells[0] = Text::from(content);
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
