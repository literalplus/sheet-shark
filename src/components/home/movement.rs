use crossterm::event::{KeyCode, KeyEvent};

use crate::components::home::state::{HomeState, TIME_ITEM_WIDTH};

pub fn is_movement(key: KeyEvent) -> bool {
    matches!(
        key.code,
        KeyCode::Left
            | KeyCode::Right
            | KeyCode::BackTab
            | KeyCode::Tab
            | KeyCode::Down
            | KeyCode::Up
    )
}

/// Returns whether a movement was made - See also [is_movement].
pub fn handle_movement(state: &mut HomeState, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Up => {
            if state.table.selected() != Some(0) {
                state.table.select_previous();
                return true;
            }
            state.ensure_column_selected();
            true
        }
        KeyCode::Down => {
            if state.table.selected() != Some(state.items.len() - 1) {
                state.table.select_next();
                return true;
            }
            state.ensure_column_selected();
            true
        }
        KeyCode::Left | KeyCode::BackTab => select_previous_column(state),
        KeyCode::Right | KeyCode::Tab => select_next_column(state),
        _ => false,
    }
}

fn select_previous_column(state: &mut HomeState) -> bool {
    state.ensure_row_selected();

    let in_first_row = state.table.selected() == Some(0);
    let in_first_column = state.table.selected_column() == Some(0);

    let want_wrap = in_first_column;
    if want_wrap {
        let can_wrap = !in_first_row;
        if can_wrap {
            state.table.select_last_column();
            state.table.select_previous();
        }
        return true;
    }

    state.table.select_previous_column();
    true
}

fn select_next_column(state: &mut HomeState) -> bool {
    state.ensure_row_selected();

    let in_last_row = state.is_last_row_selected();
    let in_last_column = state.is_last_column_selected();
    let in_penultimate_column = state.table.selected_column() == Some(TIME_ITEM_WIDTH - 2);

    // UX feature: Since duration of this entry and time of the next entry represent the same information,
    // we skip the duration. It's usually more ergonomic to enter the time explicitly. If the user wants
    // to enter a duration instead, they can move left again. That use-case is also why this feature is
    // NOT implemented in the opposite direction.
    let should_skip_duration = in_penultimate_column && !in_last_row;
    if should_skip_duration {
        state.table.select_first_column();
        state.table.select_next();
        return true;
    }

    let want_wrap = in_last_column;
    if want_wrap {
        let can_wrap = !in_last_row;
        if can_wrap {
            state.table.select_first_column();
            state.table.select_next();
        }
        return true;
    }

    state.table.select_next_column();
    true
}
