use crate::{
    components::home::{
        EditModeBehavior, Home,
        editing::EditMode,
        state::{TIME_ITEM_WIDTH, TimeItem},
    },
    layout::LayoutSlot,
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

    let ticket = Some(2);
    let suggestion = &mut state.tickets_suggestion;
    if state.table.selected_column() == ticket && suggestion.is_active() {
        let popup = suggestion.as_popup(&state.table, TABLE_WIDTHS);
        frame.render_widget(popup, area);
    }

    Ok(())
}

fn render_frame(home: &mut Home, frame: &mut Frame, area: Rect) -> Result<Rect> {
    let area = crate::layout::main_vert(LayoutSlot::MainCanvas, area);

    let block = Block::new()
        .borders(!Borders::BOTTOM)
        .border_type(BorderType::Rounded)
        .title(home.day.format(TITLE_FORMAT)?);

    frame.render_widget(&block, area);
    Ok(block.inner(area))
}

fn draw_table<'a>(
    items: &'a [TimeItem],
    selected_idx: Option<usize>,
    edit_mode: &Option<EditMode>,
) -> Table<'a> {
    let rows = items
        .iter()
        .enumerate()
        .map(draw_item(selected_idx, edit_mode));

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
) -> impl Fn((usize, &TimeItem)) -> Row {
    move |(i, item)| -> Row {
        let is_selected = Some(i) == selected_idx;
        let row = if is_selected && let Some(edit_mode) = edit_mode {
            edit_mode.style_selected_item(item)
        } else {
            item.as_row()
        };
        zebra_stripe(i, row)
    }
}

fn zebra_stripe(i: usize, row: Row) -> Row {
    let alternating_color = match i % 2 {
        0 => tailwind::SLATE.c800,
        _ => tailwind::SLATE.c900,
    };
    row.style(Style::new().bg(alternating_color))
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
