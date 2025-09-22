use time::Date;

use crate::shared::summary::TimesheetSummary;
use color_eyre::Result;

mod jira;

pub fn export(day: Date, summary: &TimesheetSummary) -> Result<()> {
    jira::export_to_jira(day, summary)
}
