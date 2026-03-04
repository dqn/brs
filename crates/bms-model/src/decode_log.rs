use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecodeLog {
    pub message: String,
    pub state: State,
}

impl DecodeLog {
    pub fn new(state: State, message: impl Into<String>) -> Self {
        DecodeLog {
            message: message.into(),
            state,
        }
    }

    pub fn get_state(&self) -> &State {
        &self.state
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Info,
    Warning,
    Error,
}
