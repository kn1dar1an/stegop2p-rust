use crossterm::event::KeyEvent;

use super::error::Error;

#[derive(Clone)]
pub enum Event {
    TerminalManagerInitialized,
    Shutdown,
    Error(Error),
    Tick,
    Render,
    Key(KeyEvent),
    CapturedInput(String),
}
