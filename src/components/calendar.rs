use std::collections::HashMap;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use educe::Educe;
use itertools::Itertools;
use lazy_static::lazy_static;
use ratatui::{
    prelude::*,
    style::palette::tailwind,
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        *,
    },
};
use time::{Date, Duration, OffsetDateTime, Weekday, ext::NumericalDuration, format_description};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Page, RelevantKey},
    layout::LayoutSlot,
    persist::{self, Command, Event, TimeEntry},
};

#[derive(Educe)]
#[educe(Default)]
pub struct Calendar {
    action_tx: Option<UnboundedSender<Action>>,
    persist_tx: Option<UnboundedSender<Command>>,
    suspended: bool,

    #[educe(Default(expression= OffsetDateTime::now_local()
            .expect("find local offset for date")
            .date()))]
    day: Date,
    days_with_timesheets: Vec<Date>,
    summary: Option<TimesheetSummary>,
}

struct TimesheetSummary {
    ticket_sums: HashMap<(String, String), Duration>,
}

impl TimesheetSummary {
    fn new(entries: Vec<TimeEntry>) -> Self {
        let ticket_sums = entries
            .into_iter()
            .into_group_map_by(|entry| {
                let project = entry
                    .project_key
                    .as_deref()
                    .unwrap_or("-")
                    .to_string();
                let ticket = entry.ticket_key.as_deref().unwrap_or("-").to_string();
                (project, ticket)
            })
            .into_iter()
            .map(|(key, group)| {
                let total_duration: Duration = group
                    .into_iter()
                    .map(|entry| Duration::minutes(entry.duration_mins as i64))
                    .sum();
                (key, total_duration)
            })
            .filter(|(_, total)| !total.is_zero())
            .collect();

        Self { ticket_sums }
    }
}

impl Component for Calendar {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_persist_handler(&mut self, tx: UnboundedSender<persist::Command>) -> Result<()> {
        self.persist_tx = Some(tx);
        Ok(())
    }

    fn is_suspended(&self) -> bool {
        self.suspended
    }

    fn init(&mut self, _area: Size) -> Result<()> {
        self.fetch_for_new_day()?;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            _ if self.handle_day_movement(key) => Ok(None),
            KeyCode::Enter => Ok(Some(Action::SetActivePage(Page::Home { day: self.day }))),
            _ => Ok(None),
        }
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

        let calendar_width = 3 * 7;
        let layout = Layout::horizontal([Constraint::Max(calendar_width + 1), Constraint::Fill(1)]);
        let [calendar_area, detail_area] = (*layout.split(area)).try_into().unwrap();
        frame.render_widget(cal, calendar_area);

        let detail_block = Block::new()
            .borders(Borders::LEFT)
            .padding(Padding::uniform(1));
        frame.render_widget(&detail_block, detail_area);
        let detail_area = detail_block.inner(detail_area);

        match &self.summary {
            Some(summary) => {
                let header = Row::new(vec!["Project", "Ticket", "Duration"]);
                let rows: Vec<Row> = summary
                    .ticket_sums
                    .iter()
                    .map(|((project, ticket), duration)| {
                        let hours = duration.whole_hours();
                        let minutes = duration.whole_minutes() % 60;
                        Row::new(vec![
                            project.clone(),
                            ticket.clone(),
                            format!("{}h {:02}m", hours, minutes),
                        ])
                    })
                    .collect();

                let table = Table::new(
                    rows,
                    [
                        Constraint::Percentage(40),
                        Constraint::Percentage(40),
                        Constraint::Percentage(20),
                    ],
                )
                .header(header.style(Style::default().add_modifier(Modifier::BOLD)))
                .block(Block::default().title("Time Summary"));

                frame.render_widget(table, detail_area);
            }
            None => {
                let text = Text::from("Loading summary...");
                frame.render_widget(text, detail_area);
            }
        }

        Ok(())
    }

    fn handle_persisted(&mut self, event: persist::Event) -> Result<Option<Action>> {
        match event {
            Event::TimesheetsOfMonthLoaded { day, timesheets } if day == self.day => {
                self.days_with_timesheets = vec![];
                let format = format_description::parse("[year]-[month]-[day]")?;
                for timesheet in timesheets {
                    if let Ok(day) = Date::parse(&timesheet.day, &format) {
                        self.days_with_timesheets.push(day);
                    }
                }
            }
            Event::TimesheetLoaded {
                day,
                timesheet: _,
                entries,
            } if day == self.day => {
                self.summary = Some(TimesheetSummary::new(entries));
            }
            _ => {}
        }
        Ok(None)
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
    fn handle_day_movement(&mut self, key: KeyEvent) -> bool {
        let new_day = match key.code {
            KeyCode::PageUp => self.day.checked_sub(Duration::days(365)),
            KeyCode::Up => self.day.checked_sub(Duration::days(7)),
            KeyCode::Left => self.day.checked_sub(Duration::days(1)),
            KeyCode::Right => self.day.checked_add(Duration::days(1)),
            KeyCode::Down => self.day.checked_add(Duration::days(7)),
            KeyCode::PageDown => self.day.checked_add(Duration::days(365)),
            _ => return false,
        }
        .expect("date math not to overflow");
        self.day = new_day;
        let _ = self.fetch_for_new_day();
        true
    }

    fn fetch_for_new_day(&mut self) -> Result<()> {
        self.persist_tx
            .as_mut()
            .expect("persist tx")
            .send(Command::LoadTimesheetsOfMonth { day: self.day })?;
        self.persist_tx
            .as_mut()
            .expect("persist tx")
            .send(Command::LoadTimesheet { day: self.day })?;
        self.days_with_timesheets = vec![];
        Ok(())
    }

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

        for day_with_timesheet in self.days_with_timesheets.iter() {
            events.add(
                *day_with_timesheet,
                Style::default().fg(tailwind::CYAN.c500),
            );
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
