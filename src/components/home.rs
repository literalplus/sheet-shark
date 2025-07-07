use color_eyre::Result;
use crossterm::event::{KeyEvent, KeyEventKind};
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};

use super::Component;
use crate::{
    action::Action,
    components::home::{
        action::HomeAction,
        editing::{EditMode, EditModeBehaviour},
        state::HomeState,
    },
    config::Config,
    layout::LayoutSlot,
};

mod editing;
mod state;
mod action {
    pub enum HomeAction {
        None,
        EnterEdit,
        ExitEdit,
        SetStatusLine(String),
    }
}

#[derive(Default)]
pub struct Home {
    config: Config,

    edit_mode: EditMode,
    state: HomeState,
}

impl Component for Home {
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }
        match self.edit_mode.handle_key_event(&mut self.state, key) {
            HomeAction::EnterEdit => {
                match self
                    .state
                    .table
                    .selected_column()
                    .and_then(|idx| EditMode::from_column_num(idx, &self.state))
                {
                    Some(it) => {
                        self.edit_mode = it;
                        return Ok(Some(Action::SetStatusLine("⌚⌚⌚".into())));
                    }
                    None => {
                        return Ok(Some(Action::SetStatusLine(
                            "Can't edit this! ⛔⛔⛔".into(),
                        )));
                    }
                }
            }
            HomeAction::ExitEdit => self.edit_mode = EditMode::default(),
            HomeAction::SetStatusLine(msg) => return Ok(Some(Action::SetStatusLine(msg))),
            HomeAction::None => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let area = crate::layout::main_vert(LayoutSlot::MainCanvas, area);

        let block = Block::new()
            .borders(!Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .title("📅 1989-12-13");
        frame.render_widget(&block, area);
        let area = block.inner(area);

        let header = ["", "Ticket", "Text", "Duration"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .height(1)
            .bg(tailwind::INDIGO.c900);
        let rows = self.state.items.iter().enumerate().map(|(i, item)| {
            let color = match i % 2 {
                0 => tailwind::SLATE.c950,
                _ => tailwind::SLATE.c900,
            };
            let row = if Some(i) == self.state.table.selected() {
                self.edit_mode.style_selected_item(item)
            } else {
                item.as_row()
            };
            row.style(Style::new().bg(color))
        });
        let table = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Length(5),
                Constraint::Max(20),
                Constraint::Fill(1),
                Constraint::Max(10),
            ],
        )
        .header(header)
        .row_highlight_style(Style::from(Modifier::REVERSED))
        .cell_highlight_style(Style::from(Modifier::BOLD));
        let table = self.edit_mode.style_table(table);

        frame.render_stateful_widget(table, area, &mut self.state.table);
        Ok(())
    }
}
