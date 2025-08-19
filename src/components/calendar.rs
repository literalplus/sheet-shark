use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use lazy_static::lazy_static;
use ratatui::{
    prelude::*,
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        *,
    },
};
use time::{Date, Duration, OffsetDateTime, Weekday, ext::NumericalDuration};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Page, RelevantKey},
    layout::LayoutSlot,
};

pub struct Calendar {
    action_tx: Option<UnboundedSender<Action>>,

    suspended: bool,
    day: Date,
}

impl Default for Calendar {
    fn default() -> Self {
        let selected_date = OffsetDateTime::now_local()
            .expect("find local offset for date")
            .date();
        Self {
            action_tx: Default::default(),
            suspended: false,
            day: selected_date,
        }
    }
}

impl Component for Calendar {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn is_suspended(&self) -> bool {
        self.suspended
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::PageUp => {
                self.day = self
                    .day
                    .checked_sub(Duration::days(365))
                    .expect("not to reach the big bang");
            }
            KeyCode::Up => {
                self.day = self
                    .day
                    .checked_sub(Duration::days(7))
                    .expect("not to reach the big bang");
            }
            KeyCode::Left => {
                self.day = self
                    .day
                    .checked_sub(Duration::days(1))
                    .expect("not to reach the big bang");
            }
            KeyCode::Right => {
                self.day = self
                    .day
                    .checked_add(Duration::days(1))
                    .expect("time not to end");
            }
            KeyCode::Down => {
                self.day = self
                    .day
                    .checked_add(Duration::days(7))
                    .expect("time not to end");
            }
            KeyCode::PageDown => {
                self.day = self
                    .day
                    .checked_add(Duration::days(365))
                    .expect("time not to end");
            }
            KeyCode::Enter => {
                return Ok(Some(Action::SetActivePage(Page::Home { day: self.day })));
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let area = crate::layout::main_vert(LayoutSlot::MainCanvas, area);

        let start = self.day;
        let block = Block::new()
            .borders(!Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .title(format!("📅 {} - Select timesheet", start.year()));
        frame.render_widget(&block, area);
        let area = block.inner(area);

        let calendar_width = 3 * 7;
        let area = Layout::horizontal([Constraint::Max(calendar_width)]).split(area)[0];

        let header_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Green);

        let default_style = Style::default().bg(Color::Rgb(50, 50, 50));

        let events = self.make_dates();
        let cal = Monthly::new(
            Date::from_calendar_date(start.year(), start.month(), 1).unwrap(),
            &events,
        )
        .show_month_header(header_style)
        .default_style(default_style);

        frame.render_widget(cal, area);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SetActivePage(Page::Calendar { day }) => {
                self.action_tx
                    .as_mut()
                    .unwrap()
                    .send(Action::SetRelevantKeys(KEYS.to_vec()))
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

impl Calendar {
    fn make_dates(&self) -> CalendarEventStore {
        let mut events = CalendarEventStore::default();
        let today = OffsetDateTime::now_local().expect("today").date();

        let first_of_month = today.replace_day(1).expect("first of month");
        let mut current_day = first_of_month;
        while current_day.month() == today.month() {
            if matches!(current_day.weekday(), Weekday::Sunday | Weekday::Saturday) {
                events.add(current_day, Style::default().dim());
            }
            current_day = current_day
                .checked_add(1.days())
                .expect("not to exceed date range");
        }

        events.add(
            today,
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Blue),
        );

        events.add(
            self.day,
            Style::default()
                .add_modifier(Modifier::UNDERLINED)
                .bg(Color::Gray),
        );

        events
    }
}

lazy_static! {
    static ref KEYS: Vec<RelevantKey> = vec![RelevantKey::new("Enter", "Select"),];
}
