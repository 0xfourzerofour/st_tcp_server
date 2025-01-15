#[derive(Debug)]
pub enum Event {
    Login(String),
    Message(String, String),
    Disconnect(String),
}
