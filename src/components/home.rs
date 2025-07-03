use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config, layout::LayoutSlot};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    items: Vec<Data>,
}

impl Default for Home {
    fn default() -> Self {
        Self { command_tx: Default::default(), config: Default::default(), items: vec![
            Data {start_time: "0915".into(), ticket: "(W)SCRUM-17".into(), text: "daily".into(), duration: "15m".into()},
            Data {start_time: "0930".into(), ticket: "(W)XAMPL-568".into(), text: "tech analysis".into(), duration: "2h".into()},
        ] }
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

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let area = crate::layout::main_vert(LayoutSlot::MainCanvas, area);

        let block = Block::new()
            .borders(!Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(2))
            .title("📅 1989-12-13");
        frame.render_widget(&block, area);
        let area = block.inner(area);

        let header = ["Star", "Ticket", "Text", "Duration"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .height(1);
        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => style::palette::tailwind::SLATE.c950,
                _ => style::palette::tailwind::SLATE.c900,
            };
            let item = data.ref_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("{content}"))))
                .collect::<Row>()
                .style(Style::new().bg(color))
        });
        let table = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Length(5),
                Constraint::Fill(3),
                Constraint::Fill(10),
                Constraint::Length(5),
            ],
        )
        .header(header);
        frame.render_widget(table, area);

        Ok(())
    }
}
