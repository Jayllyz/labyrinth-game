use crate::utils::{print_log, Color};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    RegisterTeam(RegisterTeam),
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

pub fn receive_message(stream: &mut TcpStream) -> Result<Message, String> {
    let mut buf_len = [0u8; 4];
    stream.read_exact(&mut buf_len).map_err(|e| format!("Failed to read message size: {}", e))?;

    let len = u32::from_le_bytes(buf_len) as usize;

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).map_err(|e| format!("Failed to read message content: {}", e))?;

    let str = String::from_utf8_lossy(&buf);
    let json: Message = match serde_json::from_str(&str) {
        Ok(msg) => msg,
        Err(e) => return Err(format!("Failed to parse JSON: {}", e)),
    };

    let sender = match stream.peer_addr() {
        Ok(addr) => format!("{}:{}", addr.ip(), addr.port()),
        Err(_) => String::from("unknown address"),
    };

    print_log(&format!("Received: {:?} from ({})", json, sender,), Color::Green);

    Ok(json)
}

pub fn send_message(stream: &mut TcpStream, msg: &Message) {
    let json = match serde_json::to_string(&msg) {
        Ok(json) => json,
        Err(e) => {
            print_log(&format!("Failed to serialize message: {}", e), Color::Red);
            return;
        }
    };
    let json_size = json.len() as u32;

    stream.write_all(&json_size.to_le_bytes()).expect("Failed to write JSON size");
    stream.write_all(json.as_bytes()).expect("Failed to write JSON content");
    stream.flush().expect("Failed to flush stream");

    let receiver = match stream.peer_addr() {
        Ok(addr) => format!("{}:{}", addr.ip(), addr.port()),
        Err(_) => String::from("unknown address"),
    };
    print_log(&format!("Sent: {:?} to ({})", msg, receiver), Color::Blue);
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
