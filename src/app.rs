use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::Rect;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::debug;

use crate::{
    action::{Action, Page},
    components::{
        Component, calendar::Calendar, fps::FpsCounter, home::Home, statusbar::StatusBar,
    },
    config::Config,
    persist,
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    persist_tx: UnboundedSender<persist::Command>,
    persisted_rx: UnboundedReceiver<persist::Event>,
}

impl App {
    pub fn new(
        tick_rate: f64,
        frame_rate: f64,
        persist_tx: UnboundedSender<persist::Command>,
        persisted_rx: UnboundedReceiver<persist::Event>,
    ) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![
                Box::new(Home::default()),
                Box::new(Calendar::default()),
                Box::new(FpsCounter::default()),
                Box::new(StatusBar::default()),
            ],
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            action_tx,
            action_rx,
            persist_tx,
            persisted_rx,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_persist_handler(self.persist_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        let action_tx = self.action_tx.clone();
        action_tx.send(Action::SetActivePage(Page::default()))?;
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_persisted().await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true); // Enabling this breaks copying text from the terminal
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                self.persisted_rx.close();
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if component.is_suspended() {
                continue;
            } else if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let action = if ctrl && (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('d')) {
            Action::Quit
        } else if ctrl && key.code == KeyCode::Char('z') {
            Action::Suspend
        } else {
            return Ok(());
        };
        self.action_tx.send(action)?;
        Ok(())
    }

    async fn handle_persisted(&mut self) -> Result<()> {
        while let Ok(event) = self.persisted_rx.try_recv() {
            debug!("Persisted: {event:?}");
            for component in self.components.iter_mut() {
                if let Some(action) = component.handle_persisted(event.clone())? {
                    self.action_tx.send(action)?;
                }
            }
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action {
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            for component in self.components.iter_mut() {
                if component.is_suspended() {
                    continue;
                } else if let Err(err) = component.draw(frame, frame.area()) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {err:?}")));
                }
            }
        })?;
        Ok(())
    }
}
