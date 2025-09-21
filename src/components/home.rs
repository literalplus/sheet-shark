use std::vec;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use educe::Educe;
use lazy_static::lazy_static;
use ratatui::prelude::*;
use time::{Date, OffsetDateTime};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Page, RelevantKey},
    components::home::{
        editing::{EditMode, EditModeBehavior},
        state::HomeState,
    },
    config::Config,
    persist,
};

mod action;
mod draw;
mod editing;
mod export;
mod key_handling;
mod movement;
mod persist_handling;
mod state;
mod item {}

#[derive(Educe)]
#[educe(Default)]
pub struct Home {
    #[educe(Default(expression = OffsetDateTime::now_local()
            .expect("find local offset for date")
            .date()))]
    day: Date,
    config: Config,
    action_tx: Option<UnboundedSender<Action>>,
    persist_tx: Option<UnboundedSender<persist::Command>>,

    edit_mode: Option<EditMode>,
    suspended: bool,
    state: HomeState,

    need_status_line_reset: bool,
}

impl Home {
    fn send_persist(&mut self, command: persist::Command) {
        self.persist_tx
            .as_ref()
            .expect("persist_tx initialised")
            .send(command)
            .expect("able to send persist msg")
    }

    fn send_action(&mut self, action: Action) {
        self.action_tx
            .as_ref()
            .expect("action_tx initialised")
            .send(action)
            .expect("able to send action msg")
    }
}

impl Component for Home {
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn register_persist_handler(&mut self, tx: UnboundedSender<persist::Command>) -> Result<()> {
        self.persist_tx = Some(tx);
        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn is_suspended(&self) -> bool {
        self.suspended
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = key_handling::handle(self, key);
        action::perform(self, action)?;
        Ok(None)
    }

    fn handle_persisted(&mut self, event: persist::Event) -> Result<Option<Action>> {
        let action = persist_handling::handle(self, event);
        action::perform(self, action)?;
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        draw::draw(self, frame, area)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SetActivePage(Page::Home { day }) => {
                self.send_persist(persist::Command::LoadTimesheet { day });
                self.action_tx
                    .as_mut()
                    .unwrap()
                    .send(Action::SetRelevantKeys(OUTSIDE_KEYS.to_vec()))
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

lazy_static! {
    static ref OUTSIDE_KEYS: Vec<RelevantKey> = vec![
        RelevantKey::new("Arrows", "Move"),
        RelevantKey::new("Esc", "Exit to calendar"),
        RelevantKey::new("e", "Export CSV"),
    ];
    static ref SELECTING_KEYS: Vec<RelevantKey> = vec![
        RelevantKey::new("Space", "Edit"),
        RelevantKey::new("s", "Split"),
        RelevantKey::new("Arrows", "Move"),
        RelevantKey::new("e", "Export CSV"),
    ];
    static ref EDITING_KEYS: Vec<RelevantKey> = vec![RelevantKey::new("^", "Clear"),];
}
