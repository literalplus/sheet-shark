use lazy_static::lazy_static;
use ratatui::layout::{Constraint, Layout, Rect};

lazy_static! {
    static ref MAIN_LAYOUT: Layout = Layout::vertical([Constraint::Min(5), Constraint::Length(2)]);
}

pub enum LayoutSlot {
    MainCanvas = 0,
    StatusBar = 1,
}

pub fn main_vert(slot: LayoutSlot, area: Rect) -> Rect {
    MAIN_LAYOUT.areas::<2>(area)[slot as usize]
}
