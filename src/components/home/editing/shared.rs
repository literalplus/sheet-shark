use crossterm::event::KeyEvent;
use enum_dispatch::enum_dispatch;
use ratatui::widgets::{Row, Table};

use crate::components::home::{
    action::HomeAction,
    state::{HomeState, TimeItem},
};

mod bufedit;
pub use bufedit::*;

mod movement;
pub use movement::*;

#[enum_dispatch]
pub trait EditModeBehavior {
    fn handle_key_event(&mut self, state: &mut HomeState, key: KeyEvent) -> HomeAction;
    fn style_table<'a>(&self, table: Table<'a>) -> Table<'a>;
    fn style_selected_item<'a>(&self, item: &'a TimeItem) -> Row<'a> {
        item.as_row()
    }
}
