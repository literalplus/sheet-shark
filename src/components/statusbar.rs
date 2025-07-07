use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Stylize,
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Padding},
};

use super::Component;

use crate::{
    action::{Action, RelevantKey},
    layout::LayoutSlot,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StatusBar {
    status_line: String,
    keys: Vec<RelevantKey>,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            status_line: "Hello, world!".to_owned(),
            keys: vec![
                RelevantKey::new("q", "Quit"),
                RelevantKey::new("Space", "Edit cell"),
            ],
        }
    }
}

impl Component for StatusBar {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SetStatusLine(msg) => self.status_line = msg,
            Action::SetRelevantKeys(keys) => self.keys = keys,
            _ => {}
        };
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let area = crate::layout::main_vert(LayoutSlot::StatusBar, area);

        let block = Block::new()
            .borders(!Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(2))
            .title(self.status_line.clone())
            .title_alignment(Alignment::Center);
        frame.render_widget(&block, area);

        let mut keys_text = Text::default();
        let mut first = true;
        for key in self.keys.iter() {
            if !first {
                keys_text.push_span("  ");
            } else {
                first = false;
            }
            keys_text.push_span(format!("<{}> ", key.key).blue().bold());
            keys_text.push_span(Span::from(key.text.clone()));
        }
        frame.render_widget(keys_text, block.inner(area));

        Ok(())
    }
}
