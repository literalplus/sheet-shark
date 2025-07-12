use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};

use super::Component;
use crate::{
    action::Action,
    components::home::{
        editing::{EditMode, EditModeBehavior},
        state::HomeState,
    },
    config::Config,
    layout::LayoutSlot,
};

mod editing;
mod key_handling;
mod movement;
mod state;
mod action {
    use color_eyre::eyre::ErrReport;

    use crate::components::home::editing::EditMode;

    #[derive(PartialEq, Eq)]
    pub enum HomeAction {
        None,
        EnterEditSpecific(Option<EditMode>),
        ExitEdit,
        SetStatusLine(String),
        SplitItemDown(usize),
    }

    impl From<ErrReport> for HomeAction {
        fn from(value: ErrReport) -> Self {
            Self::SetStatusLine(format!("{value}"))
        }
    }
}

#[derive(Default)]
pub struct Home {
    config: Config,

    edit_mode: Option<EditMode>,
    state: HomeState,

    need_status_line_reset: bool,
}

impl Component for Home {
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        key_handling::handle(self, key)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let area = crate::layout::main_vert(LayoutSlot::MainCanvas, area);

        let block = Block::new()
            .borders(!Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .title("📅 1989-12-13");
        frame.render_widget(&block, area);
        let area = block.inner(area);

        let header = ["#", "Ticket", "teXt", "Duration"]
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
        Ok(())
    }
}
