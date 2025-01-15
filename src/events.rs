use std::fmt::Display;

#[derive(Debug)]
pub enum Event {
    Login(String),
    Message(String, String),
    Disconnect(String),
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Event::Login(user) => writeln!(f, "LOGIN:{}", user),
            Event::Message(sender, content) => writeln!(f, "MESSAGE:{} {}", sender, content),
            Event::Disconnect(user) => writeln!(f, "LOGOUT:{}", user),
        }
    }
}
