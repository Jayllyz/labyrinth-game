use crate::logger::Logger;
use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum GameError {
    ConnectionError(io::Error),
    RegistrationError(String),
    SubscriptionError(String),
    MessageError(String),
    ChallengeError(String),
    ThreadError(String),
    RadarError(String),
    ActionError { action_type: String, details: String },
    SerializationError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::ConnectionError(err) => write!(f, "Connection error: {}", err),
            GameError::RegistrationError(msg) => write!(f, "Registration error: {}", msg),
            GameError::SubscriptionError(msg) => write!(f, "Subscription error: {}", msg),
            GameError::MessageError(msg) => write!(f, "Message error: {}", msg),
            GameError::ChallengeError(msg) => write!(f, "Challenge error: {}", msg),
            GameError::ThreadError(msg) => write!(f, "Thread error: {}", msg),
            GameError::RadarError(msg) => write!(f, "Radar error: {}", msg),
            GameError::ActionError { action_type, details } => {
                write!(f, "Action error ({}): {}", action_type, details)
            }
            GameError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for GameError {}

impl From<io::Error> for GameError {
    fn from(err: io::Error) -> Self {
        GameError::ConnectionError(err)
    }
}

impl From<serde_json::Error> for GameError {
    fn from(err: serde_json::Error) -> Self {
        GameError::SerializationError(err.to_string())
    }
}

impl GameError {
    pub fn log_error(&self, logger: &Logger) {
        logger.error(&self.to_string());
    }
}

pub type GameResult<T> = Result<T, GameError>;
