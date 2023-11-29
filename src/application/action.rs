use super::error::Error;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Action {
    Start,
    Stop,
    Tick,
    Render,
    Error(Error),
    HandleOutgoing(String),
}
