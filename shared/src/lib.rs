use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    ChatMessage(String),
    LoginResponse(String),
}
