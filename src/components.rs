use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Rect, Size},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, config::Config, persist, tui::Event};

pub mod calendar;
pub mod fps;
pub mod home;
pub mod statusbar;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
pub trait Component {
    /// Register an action handler that can send actions for processing if necessary.
    fn register_action_handler(&mut self, _tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }
    /// Register a command handler that can be used to communicate with the persistence layer.
    fn register_persist_handler(&mut self, _tx: UnboundedSender<persist::Command>) -> Result<()> {
        Ok(())
    }
    /// Register a configuration handler that provides configuration settings if necessary.
    fn register_config_handler(&mut self, _config: Config) -> Result<()> {
        Ok(())
    }
    /// Initialize the component with a specified area if necessary.
    fn init(&mut self, _area: Size) -> Result<()> {
        Ok(())
    }
    /// Whether the component is suspended, i.e. should not be rendered and should not receive events.
    /// This is handled upstream and the component does not need to check again.
    fn is_suspended(&self) -> bool {
        false
    }
    /// Handle incoming events and produce actions if necessary.
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let action = match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
            _ => None,
        };
        Ok(action)
    }
    /// Handle key events and produce actions if necessary.
    fn handle_key_event(&mut self, _key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    /// Handle mouse events and produce actions if necessary.
    fn handle_mouse_event(&mut self, _mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    /// Handle incoming events and produce actions if necessary.
    fn handle_persisted(&mut self, _event: persist::Event) -> Result<Option<Action>> {
        Ok(None)
    }
    /// Update the state of the component based on a received action. (REQUIRED)
    fn update(&mut self, _action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    /// Render the component on the screen. (REQUIRED)
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}
