use std::fmt;

#[derive(Debug)]
pub enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

impl Message {
    pub fn user_joined(username: impl Into<String>) -> Self {
        Message::UserJoined(format!("{} joined the chat", username.into()))
    }
    pub fn user_left(username: impl Into<String>) -> Self {
        Message::UserLeft(format!("{} left the chat", username.into()))
    }
    pub fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Message::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::UserJoined(username) => write!(f, "[{}]", username),
            Message::UserLeft(username) => write!(f, "[{}, :(]", username),
            Message::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}
