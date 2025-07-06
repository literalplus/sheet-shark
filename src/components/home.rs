use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config, layout::LayoutSlot};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    table_state: TableState,
    items: Vec<Data>,
}

impl Default for Home {
    fn default() -> Self {
        Self {
            command_tx: Default::default(),
            config: Default::default(),
            table_state: TableState::default(),
            items: vec![
                Data {
                    start_time: "0915".into(),
                    ticket: "(W)SCRUM-17".into(),
                    text: "daily".into(),
                    duration: "15m".into(),
                },
                Data {
                    start_time: "0930".into(),
                    ticket: "(W)XAMPL-568".into(),
                    text: "tech analysis".into(),
                    duration: "2h".into(),
                },
            ],
        }
    }
}

impl Home {
    fn next_row(&mut self) {
        self.table_state.select_next();
        //self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    fn previous_row(&mut self) {
        self.table_state.select_previous();
        //self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn next_column(&mut self) {
        self.table_state.select_next_column();
    }

    pub fn previous_column(&mut self) {
        self.table_state.select_previous_column();
    }
}

struct Data {
    pub start_time: String,
    pub ticket: String,
    pub text: String,
    pub duration: String,
}

impl Data {
    const fn ref_array(&self) -> [&String; 4] {
        [&self.start_time, &self.ticket, &self.text, &self.duration]
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Down => self.next_row(),
                KeyCode::Up => self.previous_row(),
                KeyCode::Left => self.previous_column(),
                KeyCode::Right => self.next_column(),
                KeyCode::Esc => self.table_state.select(None),
                _ => {}
            }
        }
        return Ok(None);
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
        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => tailwind::SLATE.c950,
                _ => tailwind::SLATE.c900,
            };
            let item = data.ref_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(content.to_string())))
                .collect::<Row>()
                .style(Style::new().bg(color))
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
        frame.render_stateful_widget(table, area, &mut self.table_state);

        Ok(())
    }
}
