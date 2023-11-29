use crossterm::event::KeyEvent;
use futures::channel::mpsc::UnboundedSender;
use ratatui::layout::Rect;
use ratatui::Frame;
use std::io::Result;

use crate::application::action::Action;
use crate::application::event::Event;

pub mod chat;
pub mod message;

pub trait Component {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }
    fn register_config_handler(&mut self) -> Result<()> {
        Ok(())
    }
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    fn handle_events(&mut self, event: Event) -> Result<Option<Action>> {
        let r = match event {
            Event::Key(key_event) => self.handle_key_events(key_event)?,
            _ => None,
        };
        Ok(r)
    }
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()>;
}
