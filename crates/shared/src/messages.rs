use crate::utils::{print_log, Color};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    RegisterTeam(RegisterTeam),
    RegisterTeamResult(RegisterTeamResult),
    SubscribePlayer(SubscribePlayer),
    SubscribePlayerResult(SubscribePlayerResult),
    RadarView(RadarView),
    Action(Action),
    ActionResult(ActionResult),
    MessageError(MessageError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeam {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RegistrationError {
    InvalidName,
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
    Err(SubscribeError),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SubscribeError {
    AlreadyRegistered,
    InvalidName,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RadarView {
    pub view: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    #[serde(rename = "MoveTo")]
    MoveTo(Direction),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionError {
    InvalidMove,
    OutOfMap,
    Blocked,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionResult {
    Ok,
    Completed,
    Err(ActionError),
}

#[derive(Debug, Clone)]
pub struct Client {
    pub player_name: String,
    pub team_name: String,
    pub address: SocketAddr,
    #[allow(dead_code)]
    pub moves_count: u32,
    #[allow(dead_code)]
    pub score: u32,
}

#[derive(Debug)]
pub struct Teams {
    pub team_name: String,
    pub players: Vec<Client>,
    pub score: i32,
}

pub fn receive_message(stream: &mut TcpStream) -> io::Result<Message> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    let mut buf_len = [0u8; 4];
    stream.read_exact(&mut buf_len)?;

    let len = u32::from_le_bytes(buf_len) as usize;
    const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB
    if len > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "message too large"));
    }

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;

    let str = String::from_utf8_lossy(&buf);
    let json: Message =
        serde_json::from_str(&str).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if let Ok(addr) = stream.peer_addr() {
        print_log(
            &format!("Received: {:?} from ({}:{})", json, addr.ip(), addr.port()),
            Color::Green,
        );
    }

    Ok(json)
}

pub fn send_message(stream: &mut TcpStream, msg: &Message) -> io::Result<()> {
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let json =
        serde_json::to_string(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let json_size = json.len() as u32;

    let mut buffer = Vec::with_capacity(4 + json.len());
    buffer.extend_from_slice(&json_size.to_le_bytes());
    buffer.extend_from_slice(json.as_bytes());

    stream.write_all(&buffer)?;
    stream.flush()?;

    if let Ok(addr) = stream.peer_addr() {
        print_log(&format!("Sent: {:?} to ({}:{})", msg, addr.ip(), addr.port()), Color::Blue);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

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
                SubscribeError::AlreadyRegistered,
            )),
            Message::RadarView(RadarView { view: "view".to_string() }),
            Message::Action(Action::MoveTo(Direction::Right)),
            Message::ActionResult(ActionResult::Ok),
            Message::ActionResult(ActionResult::Completed),
            Message::ActionResult(ActionResult::Err(ActionError::InvalidMove)),
            Message::ActionResult(ActionResult::Err(ActionError::OutOfMap)),
            Message::ActionResult(ActionResult::Err(ActionError::Blocked)),
            Message::MessageError(MessageError { message: "error".to_string() }),
        ];

        for msg in messages {
            let serialized = serde_json::to_string(&msg).unwrap();
            let deserialized: Message = serde_json::from_str(&serialized).unwrap();
            matches!(deserialized, |Message::RegisterTeam(_)| Message::SubscribePlayer(_)
                | Message::SubscribePlayerResult(_)
                | Message::RadarView(_)
                | Message::Action(_)
                | Message::ActionResult(_)
                | Message::MessageError(_));
        }
    }
}
