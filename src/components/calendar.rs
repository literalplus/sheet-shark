use std::{collections::HashMap, sync::Mutex};

use color_eyre::{Result, eyre::Context};
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent};
use educe::Educe;
use lazy_static::lazy_static;
use ratatui::{
    prelude::*,
    style::palette::tailwind,
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        *,
    },
};
use serde::Serialize;
use time::{Date, Duration, OffsetDateTime, Weekday, ext::NumericalDuration, format_description};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Page, RelevantKey},
    config::Config,
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

#[derive(Serialize)]
struct ProjectSummary {
    internal_name: Option<String>,
    ticket_sums: HashMap<String, Duration>,
}

#[derive(Serialize)]
struct TimesheetSummary {
    projects: HashMap<String, ProjectSummary>,
}

impl TimesheetSummary {
    fn new(entries: Vec<TimeEntry>) -> Self {
        let config = Config::get();
        let mut projects: HashMap<String, ProjectSummary> = HashMap::new();

        for entry in entries {
            let duration = Duration::minutes(entry.duration_mins as i64);
            if duration.is_zero() {
                continue;
            }

            let project_key = &entry.project_key;
            let ticket = entry.ticket_key.as_deref().unwrap_or("-").to_string();

            let project_summary = projects
                .entry(project_key.clone())
                .or_insert_with(|| Self::create_project_summary(project_key, config));

            *project_summary
                .ticket_sums
                .entry(ticket)
                .or_insert(Duration::ZERO) += duration;
        }

        Self { projects }
    }

    fn create_project_summary(project_key: &str, config: &Config) -> ProjectSummary {
        let internal_name = config
            .projects
            .get(project_key)
            .map(|p| p.internal_name.clone());

        ProjectSummary {
            internal_name,
            ticket_sums: HashMap::new(),
        }
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
            KeyCode::Char('c') => {
                let json =
                    serde_json::to_string(&self.summary).context("seralizing timesheet summary")?;
                let mut clip = CLIPBOARD.lock().expect("clipboard mutex not poisoned");
                match clip.set_contents(json) {
                    Ok(_) => Ok(Some(Action::SetStatusLine("Summary copied!".into()))),
                    Err(_) => Ok(Some(Action::SetStatusLine("Failed to copy".into()))),
                }
            }
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
            .padding(Padding::horizontal(1));
        frame.render_widget(&detail_block, detail_area);
        let detail_area = detail_block.inner(detail_area);

        match &self.summary {
            Some(summary) => {
                let header = Row::new(vec!["Project", "Ticket", "Duration"])
                    .style(Style::new().bg(tailwind::LIME.c500));
                let data_rows: Vec<_> = summary
                    .projects
                    .iter()
                    .flat_map(|(project_key, project_summary)| {
                        project_summary
                            .ticket_sums
                            .iter()
                            .map(move |(ticket, duration)| {
                                let hours = duration.whole_hours();
                                let minutes = duration.whole_minutes() % 60;
                                let display_name =
                                    project_summary.internal_name.as_deref().unwrap_or("❔");
                                Row::new(vec![
                                    format!("{} ({})", display_name, project_key),
                                    ticket.clone(),
                                    format!("{}h {:02}m", hours, minutes),
                                ])
                            })
                    })
                    .collect();

                let rows: Vec<Row> = data_rows.into_iter().collect();

                // Calculate total duration excluding "Pause" projects
                let total_duration: Duration = summary
                    .projects
                    .iter()
                    .filter(|(_, project_summary)| {
                        project_summary.internal_name.as_deref() != Some("Pause")
                    })
                    .flat_map(|(_, project_summary)| project_summary.ticket_sums.values())
                    .sum();

                let total_hours = total_duration.whole_hours();
                let total_minutes = total_duration.whole_minutes() % 60;

                // Table constraints for layout calculation
                let constraints = [
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                ];

                let table = Table::new(rows, constraints).header(header);

                // Split area for table and total
                let layout = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(1), // Space for total line
                ]);
                let areas = layout.split(detail_area);
                let table_area = areas[0];
                let total_area = areas[1];

                frame.render_widget(table, table_area);

                // Right-aligned total at the bottom
                let total_text = Paragraph::new(format!(
                    "Working time: {total_hours}h {total_minutes:02}m"
                ))
                .style(Style::new().italic())
                .alignment(Alignment::Right);
                frame.render_widget(total_text, total_area);
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
    static ref KEYS: Vec<RelevantKey> = vec![
        RelevantKey::new("Enter", "Select"),
        RelevantKey::new("c", "Copy summary"),
    ];
    static ref CLIPBOARD: Mutex<ClipboardContext> = ClipboardContext::new()
        .expect("init clipboard context")
        .into();
}
