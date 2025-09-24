use crate::{
    components::home::{
        EditModeBehavior, Home,
        editing::EditMode,
        state::{TIME_ITEM_WIDTH, TimeItem},
    },
    layout::LayoutSlot,
    shared::BREAK_PROJECT_KEY,
};
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style, Stylize, palette::tailwind},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};
use time::{format_description::FormatItem, macros::format_description};

pub(super) fn draw(home: &mut Home, frame: &mut Frame, area: Rect) -> Result<()> {
    let area = render_frame(home, frame, area)?;
    let state = &mut home.state;

    let selected_idx = state.table.selected();
    let table = draw_table(&state.items, selected_idx, &home.edit_mode);
    frame.render_stateful_widget(table, area, &mut state.table);

    if let Some(edit_mode) = &mut home.edit_mode
        && let Some(popup) = edit_mode.draw_popup(&state.table, TABLE_WIDTHS)
    {
        frame.render_widget(popup, area);
    }

    Ok(())
}

fn render_frame(home: &mut Home, frame: &mut Frame, area: Rect) -> Result<Rect> {
    let area = crate::layout::main_vert(LayoutSlot::MainCanvas, area);

    let total_hours = home.total_working_hours();
    let title = if total_hours.is_zero() {
        home.day.format(TITLE_FORMAT)?
    } else {
        format!(
            "{} - {}h{}m",
            home.day.format(TITLE_FORMAT)?,
            total_hours.whole_hours(),
            total_hours.whole_minutes() % 60
        )
    };

    let block = Block::new()
        .borders(!Borders::BOTTOM)
        .border_type(BorderType::Rounded)
        .title(title);

    frame.render_widget(&block, area);
    Ok(block.inner(area))
}

fn draw_table<'a>(
    items: &'a [TimeItem],
    selected_idx: Option<usize>,
    edit_mode: &Option<EditMode>,
) -> Table<'a> {
    let mismatching_idxs = mark_mismatching_items(items);
    let rows = items
        .iter()
        .enumerate()
        .map(draw_item(selected_idx, edit_mode, &mismatching_idxs));

    let header = TABLE_HEADERS
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .height(1)
        .bg(tailwind::INDIGO.c900);

    let table = Table::new(rows, TABLE_WIDTHS)
        .header(header)
        .row_highlight_style(Style::from(Modifier::REVERSED))
        .cell_highlight_style(
            Style::from(Modifier::BOLD)
                .not_reversed()
                .bg(tailwind::SLATE.c400),
        );

    match edit_mode {
        Some(edit_mode) => edit_mode.style_table(table),
        None => table,
    }
}

fn draw_item(
    selected_idx: Option<usize>,
    edit_mode: &Option<EditMode>,
    mismatching_idxs: &[usize],
) -> impl Fn((usize, &TimeItem)) -> Row {
    move |(i, item)| -> Row {
        let is_selected = Some(i) == selected_idx;
        if is_selected && let Some(edit_mode) = edit_mode {
            edit_mode.style_selected_item(item)
        } else {
            create_row_for_item(i, item, mismatching_idxs.contains(&i))
        }
    }
}

fn create_row_for_item(i: usize, item: &TimeItem, is_mismatch: bool) -> Row<'_> {
    if item.project == BREAK_PROJECT_KEY {
        let mut cells = item.as_cells(is_mismatch);
        cells[2] = "🏖️🏖️🏖️".into();
        Row::new(cells).bg(tailwind::EMERALD.c900)
    } else {
        zebra_stripe(i, item.as_row(is_mismatch))
    }
}

fn zebra_stripe(i: usize, row: Row) -> Row {
    let alternating_color = match i % 2 {
        0 => tailwind::SLATE.c800,
        _ => tailwind::SLATE.c900,
    };
    row.style(Style::new().bg(alternating_color))
}

pub fn mark_mismatching_items(items: &[TimeItem]) -> Vec<usize> {
    let mut mismatching_indices = Vec::new();

    for (i, current_item) in items.iter().enumerate() {
        let Some(next_item) = items.get(i + 1) else {
            break;
        };

        let expected_next_start_time = current_item.next_start_time();
        let actual_next_start_time = next_item.start_time;

        if expected_next_start_time != actual_next_start_time {
            mismatching_indices.push(i);
        }
    }

    mismatching_indices
}

const TITLE_FORMAT: &[FormatItem<'static>] =
    format_description!("📅 [weekday], [year]-[month]-[day] (KW [week_number])");

const TABLE_WIDTHS: [Constraint; TIME_ITEM_WIDTH] = [
    // + 1 is for padding.
    Constraint::Length(5),
    Constraint::Length(3),
    Constraint::Max(20),
    Constraint::Fill(1),
    Constraint::Max(10),
];
const TABLE_HEADERS: [&str; TIME_ITEM_WIDTH] = ["#", "", "Ticket", "Description", "Duration"];
