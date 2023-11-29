use core::fmt::Display;

use uuid::Uuid;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Sender {
    Incoming(String),
    Outgoing,
    System,
}

impl Display for Sender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sender::Incoming(from) => write!(f, "{}", from),
            Sender::Outgoing => write!(f, "You"),
            Sender::System => write!(f, "System"),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Message {
    uuid: Uuid,
    message: String,
    sender: Sender,
}

impl Message {
    pub fn new(message: String, sender: Sender) -> Self {
        let uuid = Uuid::new_v4();
        Self { uuid, message, sender }
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        format!("{}> {}", self.sender, self.message)
    }
}
