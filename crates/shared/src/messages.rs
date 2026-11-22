use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read, Write},
    net::SocketAddr,
};

use crate::{
    errors::{GameError, GameResult},
    logger::Logger,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    RegisterTeam(RegisterTeam),
    RegisterTeamResult(RegisterTeamResult),
    SubscribePlayer(SubscribePlayer),
    SubscribePlayerResult(SubscribePlayerResult),
    RadarView(RadarView),
    Action(Action),
    ActionError(ActionError),
    MessageError(MessageError),
    Hint(Hint),
    Challenge(Challenge),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeam {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RegistrationError {
    InvalidName,
    TeamAlreadyRegistered,
    AlreadyRegistered,
    TooManyPlayers,
    InvalidRegistrationToken,
    ServerError,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RegisterTeamResult {
    Ok { expected_players: u8, registration_token: String },
    Err(RegistrationError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageError {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribePlayer {
    pub name: String,
    pub registration_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SubscribePlayerResult {
    Ok,
    Err(RegistrationError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RadarView(pub String);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Right,
    Left,
    Front,
    Back,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    MoveTo(Direction),
    SolveChallenge { answer: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionError {
    InvalidMove,
    OutOfMap,
    Blocked,
    InvalidChallengeSolution,
    SolveChallengeFirst,
    CannotPassThroughOpponent,
    CannotPassThroughWall,
    NoRunningChallenge,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Hint {
    RelativeCompass { angle: f32 },
    GridSize { columns: u32, rows: u32 },
    Secret(u128),
    SOS,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Challenge {
    SecretSumModulo(u128),
}

#[derive(Debug, Clone)]
pub struct Client {
    pub player_name: String,
    pub team_name: String,
    pub address: SocketAddr,
    pub registration_token: String,
}

#[derive(Debug, Clone)]
pub struct Teams {
    pub team_name: String,
    pub players: Vec<Client>,
    pub score: i32,
    pub registration_token: String,
    pub max_players: u8,
}

pub fn receive_message(stream: &mut impl Read) -> GameResult<Message> {
    let mut buf_len = [0u8; 4];
    stream.read_exact(&mut buf_len).map_err(|e| match e.kind() {
        io::ErrorKind::UnexpectedEof => GameError::MessageError("Incomplete message length".into()),
        _ => GameError::ConnectionError(e),
    })?;

    let len = u32::from_le_bytes(buf_len) as usize;
    const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB
    if len > MAX_MESSAGE_SIZE {
        return Err(GameError::MessageError("Message too large".into()));
    }

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).map_err(GameError::ConnectionError)?;

    let str = match String::from_utf8(buf) {
        Ok(s) => s,
        Err(e) => return Err(GameError::MessageError(format!("Invalid UTF-8: {}", e))),
    };

    serde_json::from_str(&str).map_err(|e| GameError::MessageError(format!("Invalid JSON: {}", e)))
}

pub fn send_message(stream: &mut impl Write, msg: &Message) -> GameResult<()> {
    let json =
        serde_json::to_string(msg).map_err(|e| GameError::SerializationError(e.to_string()))?;

    if let Message::Action(Action::SolveChallenge { answer }) = msg {
        Logger::get_instance()
            .debug(&format!("Sending message: Action::SolveChallenge {{ answer: {} }}", answer));
    }

    let len = json.len();
    let mut buffer = Vec::with_capacity(4 + len);
    buffer.extend_from_slice(&(len as u32).to_le_bytes());
    buffer.extend_from_slice(json.as_bytes());

    stream.write_all(&buffer).and_then(|_| stream.flush()).map_err(GameError::ConnectionError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::io::Cursor;

    #[test]
    fn test_all_messages() {
        let messages = vec![
            Message::RegisterTeam(RegisterTeam { name: "team1".to_string() }),
            Message::SubscribePlayer(SubscribePlayer {
                name: "player1".to_string(),
                registration_token: "token".to_string(),
            }),
            Message::SubscribePlayerResult(SubscribePlayerResult::Ok),
            Message::SubscribePlayerResult(SubscribePlayerResult::Err(
                RegistrationError::AlreadyRegistered,
            )),
            Message::RadarView(RadarView("radar".to_string())),
            Message::Action(Action::MoveTo(Direction::Right)),
            Message::Action(Action::SolveChallenge { answer: "answer".to_string() }),
            Message::MessageError(MessageError { message: "error".to_string() }),
        ];

        for msg in messages {
            let serialized = serde_json::to_string(&msg).unwrap();
            let deserialized: Message = serde_json::from_str(&serialized).unwrap();
            assert_eq!(serde_json::to_string(&deserialized).unwrap(), serialized);
        }
    }

    fn frame(len: u32, body: &[u8]) -> Cursor<Vec<u8>> {
        let mut bytes = len.to_le_bytes().to_vec();
        bytes.extend_from_slice(body);
        Cursor::new(bytes)
    }

    #[test]
    fn test_send_then_receive_round_trips() {
        let msg = Message::Action(Action::MoveTo(Direction::Left));
        let mut buf = Vec::new();
        send_message(&mut buf, &msg).unwrap();

        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(&buf[..4], &(json.len() as u32).to_le_bytes());

        let received = receive_message(&mut Cursor::new(buf)).unwrap();
        assert_eq!(serde_json::to_string(&received).unwrap(), json);
    }

    #[test]
    fn test_receive_rejects_oversized_message() {
        let err = receive_message(&mut frame(1024 * 1024 + 1, b"")).unwrap_err();
        assert!(matches!(err, GameError::MessageError(m) if m == "Message too large"));
    }

    #[test]
    fn test_receive_rejects_truncated_length() {
        let err = receive_message(&mut Cursor::new(vec![1, 0])).unwrap_err();
        assert!(matches!(err, GameError::MessageError(m) if m == "Incomplete message length"));
    }

    #[test]
    fn test_receive_rejects_truncated_body() {
        let err = receive_message(&mut frame(10, b"{}")).unwrap_err();
        assert!(matches!(err, GameError::ConnectionError(_)));
    }

    #[test]
    fn test_receive_rejects_invalid_utf8_and_json() {
        let err = receive_message(&mut frame(2, &[0xff, 0xfe])).unwrap_err();
        assert!(matches!(err, GameError::MessageError(m) if m.starts_with("Invalid UTF-8")));

        let err = receive_message(&mut frame(3, b"{x}")).unwrap_err();
        assert!(matches!(err, GameError::MessageError(m) if m.starts_with("Invalid JSON")));
    }
}
