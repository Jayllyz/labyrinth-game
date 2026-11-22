use std::io;

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] io::Error),
    #[error("Team registration error: {0}")]
    TeamRegistrationError(String),
    #[error("Agent subscription error: {0}")]
    AgentSubscriptionError(String),
    #[error("Message error: {0}")]
    MessageError(String),
    #[error("Thread error: {0}")]
    ThreadError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<serde_json::Error> for GameError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

pub type GameResult<T> = Result<T, GameError>;

#[cfg(test)]
mod tests {
    use super::*;
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
        match success {
            Ok(value) => assert_eq!(value, 42),
            Err(_) => panic!("Expected Ok(42)"),
        }

        let failure: GameResult<i32> = Err(GameError::MessageError("test error".to_string()));
        assert!(failure.is_err());
        assert!(matches!(failure, Err(GameError::MessageError(_))));
    }

    #[test]
    fn test_error_debug_impl() {
        let error = GameError::MessageError("test error".to_string());
        let debug_string = format!("{error:?}");
        assert!(debug_string.contains("MessageError"));
        assert!(debug_string.contains("test error"));
    }
}
