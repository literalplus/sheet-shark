pub mod table_popup {
    use itertools::Itertools;
    use ratatui::{
        prelude::*,
        style::palette::tailwind::{INDIGO, SLATE},
        widgets::{
            Block, BorderType, Clear, List, ListItem, ListState, Padding, TableState, Widget,
        },
    };

    const ASSUMED_SPACING: u16 = 1;
    const ASSUMED_HEADER_HEIGHT: u16 = 1;

    pub struct TablePopup<'a> {
        table_state: &'a TableState,
        list_state: &'a mut ListState,
        items: Vec<ListItem<'a>>,
        constraints: Vec<Constraint>,
    }

    impl<'a> TablePopup<'a> {
        pub fn new<CI>(
            table_state: &'a TableState,
            list_state: &'a mut ListState,
            items: &'a [ListItem<'a>],
            constraints: CI,
        ) -> Self
        where
            CI: IntoIterator<Item = Constraint>,
        {
            Self {
                table_state,
                list_state,
                items: items.into(),
                constraints: constraints.into_iter().collect_vec(),
            }
        }

        fn find_best_area(&self, constraints: &[Constraint], area: Rect) -> Option<Rect> {
            let (row_idx, col_idx) = self.table_state.selected_cell()?;
            let height_above = row_idx as u16;

            let column_rect = Layout::horizontal(constraints)
                .spacing(ASSUMED_SPACING)
                .split(area)[col_idx];
            let above_and_below = Layout::vertical([
                Constraint::Length(ASSUMED_HEADER_HEIGHT),
                Constraint::Length(height_above),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(column_rect);

            let height_below = above_and_below[3].height;
            let best = if height_below > height_above {
                above_and_below[3]
            } else {
                above_and_below[1]
            };
            Some(best)
        }
    }

    impl Widget for TablePopup<'_> {
        fn render(self, area: Rect, buf: &mut Buffer)
        where
            Self: Sized,
        {
            let Some(area) = self.find_best_area(&self.constraints, area) else {
                return;
            };

            Clear.render(area, buf);

            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1))
                .style(Style::new().bg(INDIGO.c950));

            let list = List::new(self.items)
                .block(block)
                .highlight_style(Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD));
            StatefulWidget::render(list, area, buf, self.list_state);
        }
    }
}
