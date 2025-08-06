use std::cmp::Ordering;

use chrono::TimeDelta;
use color_eyre::eyre::{Result, bail, eyre};
use crossterm::event::KeyEvent;
use humantime::parse_duration;
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
pub struct Duration {
    buf: BufEditBehavior,
}

impl Duration {
    fn handle_save(&mut self, state: &mut HomeState) -> Result<()> {
        if self.buf.parse::<u16>().is_ok() {
            self.buf.push('m');
        }

        let parsed = parse_duration(&self.buf).map_err(|err| eyre!("Invalid: {err}"))?;
        if parsed.as_secs() == 0 || parsed.as_secs() % 60 != 0 {
            bail!("Duration must be a whole number of minutes (e.g. 15m)");
        }

        state.expect_selected_item_mut().duration = parsed;

        if state.is_last_row_selected() {
            Self::create_next_item(state);
        } else {
            // The idea behind editing duration is that it's taken from (or added to) the next item(s)
            Self::adjust_following_items(state);
        }

        Ok(())
    }

    fn create_next_item(state: &mut HomeState) {
        let new_item = TimeItem::new(
            Default::default(),
            state.expect_selected_item().next_start_time(),
        );
        state.items.push(new_item);
        state.table.select_last();
        state.table.select_first_column();
    }

    fn adjust_following_items(state: &mut HomeState) {
        let my_index = state.table.selected().expect("selected");
        let last_index = state.items.len() - 1;
        let new_end_time = state.expect_selected_item().next_start_time();
        let mut num_items_to_remove = 0;

        for (idx, item_to_adjust) in state.items.iter_mut().enumerate().skip(my_index + 1) {
            if idx == last_index && item_to_adjust.duration.is_zero() {
                item_to_adjust.start_time = new_end_time;
                item_to_adjust.version.touch();
                break; // duration hasn't been filled yet
            }

            let delta = new_end_time - item_to_adjust.start_time;
            let delta_duration_abs =
                std::time::Duration::from_secs(delta.num_seconds().unsigned_abs());

            if delta.is_zero() {
                break;
            } else if delta < TimeDelta::zero() {
                item_to_adjust.start_time += delta; // Actually subtracts; delta < 0
                item_to_adjust.duration += delta_duration_abs;
                item_to_adjust.version.touch();
                break;
            }

            // Otherwise, we have to steal from the next item(s)
            let big_enough_to_cover = item_to_adjust.duration.cmp(&delta_duration_abs);
            match big_enough_to_cover {
                Ordering::Equal | Ordering::Less => num_items_to_remove += 1,
                Ordering::Greater => {
                    item_to_adjust.duration -= delta_duration_abs;
                    item_to_adjust.start_time = new_end_time;
                    item_to_adjust.version.touch();
                    break;
                }
            }
        }

        if num_items_to_remove > 0 {
            let drain_start = my_index + 1;
            state.drain_items(drain_start..(drain_start + num_items_to_remove));
        }
    }
}

impl EditModeBehavior for Duration {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction {
        if self.buf.should_save(key)
            && let Err(err) = self.handle_save(state)
        {
            return err.into();
        }
        self.buf.handle_key_event(state, key)
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
        table.cell_highlight_style(
            Style::from(Modifier::ITALIC)
                .not_reversed()
                .bg(tailwind::INDIGO.c300),
        )
    }
}
