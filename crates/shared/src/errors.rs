use crate::logger::Logger;
use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum GameError {
    ConnectionError(io::Error),
    TeamRegistrationError(String),
    AgentSubscriptionError(String),
    MessageError(String),
    ThreadError(String),
    SerializationError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::ConnectionError(err) => write!(f, "Connection error: {}", err),
            GameError::TeamRegistrationError(msg) => write!(f, "Team registration error: {}", msg),
            GameError::AgentSubscriptionError(msg) => {
                write!(f, "Agent subscription error: {}", msg)
            }
            GameError::MessageError(msg) => write!(f, "Message error: {}", msg),
            GameError::ThreadError(msg) => write!(f, "Thread error: {}", msg),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::Logger;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_error_display() {
        let err = GameError::ConnectionError(IoError::new(
            ErrorKind::ConnectionRefused,
            "connection refused",
        ));
        assert_eq!(err.to_string(), "Connection error: connection refused");

        let err = GameError::TeamRegistrationError("team already exists".to_string());
        assert_eq!(err.to_string(), "Team registration error: team already exists");

        let err = GameError::AgentSubscriptionError("invalid agent".to_string());
        assert_eq!(err.to_string(), "Agent subscription error: invalid agent");

        let err = GameError::MessageError("invalid format".to_string());
        assert_eq!(err.to_string(), "Message error: invalid format");

        let err = GameError::ThreadError("thread panic".to_string());
        assert_eq!(err.to_string(), "Thread error: thread panic");

        let err = GameError::SerializationError("invalid JSON".to_string());
        assert_eq!(err.to_string(), "Serialization error: invalid JSON");
    }

    #[test]
    fn test_error_conversion_from_io_error() {
        let io_error = IoError::new(ErrorKind::NotFound, "file not found");
        let game_error = GameError::from(io_error);

        match game_error {
            GameError::ConnectionError(err) => {
                assert_eq!(err.kind(), ErrorKind::NotFound);
                assert_eq!(err.to_string(), "file not found");
            }
            _ => panic!("Expected ConnectionError variant"),
        }
    }

    #[test]
    fn test_error_conversion_from_serde_error() {
        let serde_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let game_error = GameError::from(serde_error);

        match game_error {
            GameError::SerializationError(msg) => {
                assert!(msg.contains("expected value"));
            }
            _ => panic!("Expected SerializationError variant"),
        }
    }

    #[test]
    fn test_game_result_type() {
        let success: GameResult<i32> = Ok(42);
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 42);

        let failure: GameResult<i32> = Err(GameError::MessageError("test error".to_string()));
        assert!(failure.is_err());
        assert!(matches!(failure.unwrap_err(), GameError::MessageError(_)));
    }

    #[test]
    fn test_error_debug_impl() {
        let error = GameError::MessageError("test error".to_string());
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("MessageError"));
        assert!(debug_string.contains("test error"));
    }

    #[test]
    fn test_error_logging() {
        Logger::init(true);
        let logger = Logger::get_instance();

        let errors = vec![
            GameError::ConnectionError(IoError::new(ErrorKind::Other, "network error")),
            GameError::TeamRegistrationError("duplicate team".to_string()),
            GameError::AgentSubscriptionError("invalid agent id".to_string()),
            GameError::MessageError("malformed message".to_string()),
            GameError::ThreadError("thread crashed".to_string()),
            GameError::SerializationError("invalid JSON format".to_string()),
        ];

        for error in errors {
            error.log_error(&logger);
        }
    }
}
