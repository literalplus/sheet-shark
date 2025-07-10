use color_eyre::eyre::{Result, bail, eyre};
use crossterm::event::KeyEvent;
use humantime::parse_duration;
use ratatui::{
    style::{Modifier, Style, Stylize, palette::tailwind},
    text::Text,
    widgets::{Row, Table},
};

use super::{EditModeBehavior};
use crate::components::home::{
    action::HomeAction, editing::shared::BufEditBehavior, state::{HomeState, TimeItem}
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
            Self::adjust_following_items(state);
        }
        Ok(())
    }

    fn create_next_item(state: &mut HomeState) {
        let new_item = TimeItem {
            start_time: state.expect_selected_item().next_start_time(),
            ..TimeItem::default()
        };
        state.items.push(new_item);
        state.table.select_last();
        state.table.select_first_column();
    }

    fn adjust_following_items(state: &mut HomeState) {
        let items_before_mine = state.table.selected().unwrap_or(0);
        let mut next_start_time = state.expect_selected_item().next_start_time();

        for item_to_adjust in state.items.iter_mut().skip(items_before_mine + 1) {
            item_to_adjust.start_time = next_start_time;
            next_start_time = item_to_adjust.next_start_time();
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
