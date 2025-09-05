use std::vec;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use educe::Educe;
use lazy_static::lazy_static;
use ratatui::{
    prelude::*,
    style::palette::tailwind::{self},
    widgets::*,
};
use time::{Date, OffsetDateTime, format_description};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Page, RelevantKey},
    components::home::{
        editing::{EditMode, EditModeBehavior},
        state::HomeState,
    },
    config::Config,
    layout::LayoutSlot,
    persist,
};

mod action;
mod editing;
mod key_handling;
mod movement;
mod persist_handling;
mod state;

#[derive(Educe)]
#[educe(Default)]
pub struct Home {
    #[educe(Default(expression = OffsetDateTime::now_local()
            .expect("find local offset for date")
            .date()))]
    day: Date,
    config: Config,
    action_tx: Option<UnboundedSender<Action>>,
    persist_tx: Option<UnboundedSender<persist::Command>>,

    edit_mode: Option<EditMode>,
    suspended: bool,
    state: HomeState,

    need_status_line_reset: bool,
}

impl Home {
    fn send_persist(&mut self, command: persist::Command) {
        self.persist_tx
            .as_ref()
            .expect("persist_tx initialised")
            .send(command)
            .expect("able to send persist msg")
    }
}

impl Component for Home {
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn register_persist_handler(&mut self, tx: UnboundedSender<persist::Command>) -> Result<()> {
        self.persist_tx = Some(tx);
        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn is_suspended(&self) -> bool {
        self.suspended
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = key_handling::handle(self, key);
        action::perform(self, action)
    }

    fn handle_persisted(&mut self, event: persist::Event) -> Result<Option<Action>> {
        let action = persist_handling::handle(self, event);
        action::perform(self, action)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let area = crate::layout::main_vert(LayoutSlot::MainCanvas, area);

        let format =
            format_description::parse("[weekday], [year]-[month]-[day] (KW [week_number])")?;
        let iso_day = self.day.format(&format)?;
        let block = Block::new()
            .borders(!Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .title(format!("📅 {iso_day}"));
        frame.render_widget(&block, area);
        let area = block.inner(area);

        let header = ["#", "", "Ticket", "Description", "Duration"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .height(1)
            .bg(tailwind::INDIGO.c900);
        let rows = self.state.items.iter().enumerate().map(|(i, item)| {
            let color = match i % 2 {
                0 => tailwind::SLATE.c800,
                _ => tailwind::SLATE.c900,
            };
            let row = if Some(i) == self.state.table.selected()
                && let Some(edit_mode) = &self.edit_mode
            {
                edit_mode.style_selected_item(item)
            } else {
                item.as_row()
            };
            row.style(Style::new().bg(color))
        });
        let widths = [
            // + 1 is for padding.
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Max(20),
            Constraint::Fill(1),
            Constraint::Max(10),
        ];
        let table = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::from(Modifier::REVERSED))
            .cell_highlight_style(
                Style::from(Modifier::BOLD)
                    .not_reversed()
                    .bg(tailwind::SLATE.c400),
            );

        let table = match &self.edit_mode {
            Some(edit_mode) => edit_mode.style_table(table),
            None => table,
        };

        frame.render_stateful_widget(table, area, &mut self.state.table);

        if self.state.table.selected_column() == Some(2) {
            let suggestion = &mut self.state.tickets_suggestion;
            let popup = suggestion.as_popup(&self.state.table, widths);
            frame.render_widget(popup, area);
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SetActivePage(Page::Home { day }) => {
                self.send_persist(persist::Command::LoadTimesheet { day });
                self.action_tx
                    .as_mut()
                    .unwrap()
                    .send(Action::SetRelevantKeys(OUTSIDE_KEYS.to_vec()))
                    .expect("sent initial keys");
                self.day = day;
                self.suspended = false;
            }
            Action::SetActivePage(_) => {
                self.suspended = true;
            }
            _ => {}
        }
        Ok(None)
    }
}

lazy_static! {
    static ref OUTSIDE_KEYS: Vec<RelevantKey> = vec![
        RelevantKey::new("Arrows", "Move"),
        RelevantKey::new("Esc", "Exit to calendar"),
    ];
    static ref SELECTING_KEYS: Vec<RelevantKey> = vec![
        RelevantKey::new("Space", "Edit"),
        RelevantKey::new("s", "Split"),
        RelevantKey::new("Arrows", "Move"),
    ];
    static ref EDITING_KEYS: Vec<RelevantKey> = vec![RelevantKey::new("^", "Clear"),];
}
