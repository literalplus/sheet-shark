use ratatui::{
    prelude::*,
    style::palette::tailwind,
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        *,
    },
};
use time::{Date, Duration, OffsetDateTime, Weekday, ext::NumericalDuration};

use crate::{components::calendar::TimesheetSummary, shared::BREAK_PROJECT_KEY};

pub struct TimesheetSummaryPanel<'a> {
    summary: &'a TimesheetSummary,
}

const TABLE_CONSTRAINTS: [Constraint; 3] = [
    Constraint::Percentage(40),
    Constraint::Percentage(40),
    Constraint::Percentage(20),
];

impl<'a> TimesheetSummaryPanel<'a> {
    pub fn new(summary: &'a TimesheetSummary) -> Self {
        Self { summary }
    }

    fn create_header(&self) -> Row<'_> {
        Row::new(vec!["Project", "Ticket", "Duration"]).style(Style::new().bg(tailwind::LIME.c500))
    }

    fn create_data_rows(&self) -> Vec<Row<'_>> {
        self.summary
            .projects
            .iter()
            .flat_map(|(project_key, project_summary)| {
                project_summary
                    .ticket_sums
                    .iter()
                    .map(move |(ticket, duration)| {
                        self.create_single_row(project_key, project_summary, ticket, duration)
                    })
            })
            .collect()
    }

    fn create_single_row(
        &self,
        project_key: &str,
        project_summary: &crate::components::calendar::ProjectSummary,
        ticket: &str,
        duration: &Duration,
    ) -> Row<'_> {
        let project_display = self.format_project_display(project_key, project_summary);
        let duration_display = self.format_duration_display(duration);

        Row::new(vec![project_display, ticket.to_string(), duration_display])
    }

    fn format_project_display(
        &self,
        project_key: &str,
        project_summary: &crate::components::calendar::ProjectSummary,
    ) -> String {
        let display_name = project_summary.internal_name.as_deref().unwrap_or("❔");
        format!("{display_name} ({project_key}) ")
    }

    fn format_duration_display(&self, duration: &Duration) -> String {
        let hours = duration.whole_hours();
        let minutes = duration.whole_minutes() % 60;

        match (hours, minutes) {
            (0, 0) => "-".to_string(),
            (0, m) => format!("{m}m"),
            (h, 0) => format!("{h}h"),
            (h, m) => format!("{h}h {m:02}m"),
        }
    }

    fn calculate_total_duration(&self) -> Duration {
        self.summary
            .projects
            .iter()
            .filter(|(_, project_summary)| {
                project_summary.internal_name.as_deref() != Some(BREAK_PROJECT_KEY)
            })
            .flat_map(|(_, project_summary)| project_summary.ticket_sums.values())
            .sum()
    }

    fn create_total_paragraph(&self, total_duration: Duration) -> Paragraph<'_> {
        let formatted_duration = self.format_duration_display(&total_duration);

        Paragraph::new(format!("Working time: {formatted_duration}"))
            .style(Style::new().italic())
            .alignment(Alignment::Right)
    }
}

impl Widget for TimesheetSummaryPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let header = self.create_header();
        let rows = self.create_data_rows();
        let total_duration = self.calculate_total_duration();

        let table = Table::new(rows, TABLE_CONSTRAINTS).header(header);

        // Split area for table and total
        let layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1), // Space for total line
        ]);
        let areas = layout.split(area);
        let table_area = areas[0];
        let total_area = areas[1];

        Widget::render(table, table_area, buf);

        let total_paragraph = self.create_total_paragraph(total_duration);
        Widget::render(total_paragraph, total_area, buf);
    }
}

pub struct TimesheetCalendar<'a> {
    day: Date,
    days_with_timesheets: &'a [Date],
    summary: Option<&'a TimesheetSummary>,
}

impl<'a> TimesheetCalendar<'a> {
    pub fn new(
        day: Date,
        days_with_timesheets: &'a [Date],
        summary: Option<&'a TimesheetSummary>,
    ) -> Self {
        Self {
            day,
            days_with_timesheets,
            summary,
        }
    }

    fn create_calendar_events(&self) -> CalendarEventStore {
        use ratatui::widgets::calendar::CalendarEventStore;

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

    fn create_calendar_widget(&self) -> Monthly<'_, CalendarEventStore> {
        let start = self.day;
        let header_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Green);

        let default_style = Style::default().bg(Color::Rgb(50, 50, 50));

        let events = self.create_calendar_events();
        Monthly::new(
            Date::from_calendar_date(start.year(), start.month(), 1).unwrap(),
            events,
        )
        .show_month_header(header_style)
        .default_style(default_style)
    }

    fn render_detail_panel(&self, area: Rect, buf: &mut Buffer) {
        if let Some(summary) = self.summary {
            let detail_panel = TimesheetSummaryPanel::new(summary);
            Widget::render(detail_panel, area, buf);
        } else {
            let text = Text::from("Loading summary...");
            Widget::render(Paragraph::new(text), area, buf);
        }
    }
}

impl Widget for TimesheetCalendar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let start = self.day;
        let block = Block::new()
            .borders(!Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .title(format!("📅 {} - Select timesheet", start.year()));
        Widget::render(&block, area, buf);
        let area = block.inner(area);

        let cal = self.create_calendar_widget();

        let calendar_width = 3 * 7;
        let layout = Layout::horizontal([Constraint::Max(calendar_width + 1), Constraint::Fill(1)]);
        let [calendar_area, detail_area] = (*layout.split(area)).try_into().unwrap();
        Widget::render(cal, calendar_area, buf);

        let detail_block = Block::new()
            .borders(Borders::LEFT)
            .padding(Padding::horizontal(1));
        Widget::render(&detail_block, detail_area, buf);
        let detail_area = detail_block.inner(detail_area);

        self.render_detail_panel(detail_area, buf);
    }
}
