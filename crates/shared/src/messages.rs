use crate::utils::{print_log, Color};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Hello,
    Welcome(Welcome),
    Subscribe(Subscribe),
    SubscribeResult(SubscribeResult),
    View(View),
    Action(Action),
    ActionResult(ActionResult),
    MessageError(MessageError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hello;

#[derive(Serialize, Deserialize, Debug)]
pub struct Welcome {
    pub version: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageError {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscribe {
    pub name: String,
    pub team: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SubscribeResult {
    Ok,
    Err(SubscribeError),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SubscribeError {
    AlreadyRegistered,
    InvalidName,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct View {
    pub view: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ViewModel {
    pub view: View,
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
    match stream.read_exact(&mut buf_len) {
        Ok(_) => (),
        Err(e) => Err(format!("Failed to read message length: {}", e))?,
    }

    let len = u32::from_be_bytes(buf_len) as usize;

    let mut buf = vec![0u8; len];
    match stream.read_exact(&mut buf) {
        Ok(_) => (),
        Err(e) => Err(format!("Failed to read message body: {}", e))?,
    };

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

pub fn send_message(stream: &mut TcpStream, msg: Message) {
    let json = serde_json::to_string(&msg).expect("Failed to serialize message");
    let len = json.len() as u32;

    stream.write_all(&len.to_be_bytes()).expect("Failed to write to stream");
    stream.write_all(json.as_bytes()).expect("Failed to write to stream");
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
    fn test_hello_message() {
        let msg = Message::Hello;
        let serialized = serde_json::to_string(&msg).unwrap();
        assert_eq!(serialized, r#""Hello""#);

        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        matches!(deserialized, Message::Hello);
    }

    #[test]
    fn test_all_messages() {
        let messages = vec![
            Message::Hello,
            Message::Welcome(Welcome { version: 1 }),
            Message::Subscribe(Subscribe {
                name: "Player1".to_string(),
                team: "Team1".to_string(),
            }),
            Message::SubscribeResult(SubscribeResult::Ok),
            Message::View(View { view: "Initial state".to_string() }),
            Message::Action(Action::MoveTo(Direction::Right)),
            Message::ActionResult(ActionResult::Ok),
            Message::MessageError(MessageError { message: "Error".to_string() }),
        ];

        for msg in messages {
            let serialized = serde_json::to_string(&msg).unwrap();
            let deserialized: Message = serde_json::from_str(&serialized).unwrap();
            matches!(
                deserialized,
                Message::Hello
                    | Message::Welcome(_)
                    | Message::Subscribe(_)
                    | Message::SubscribeResult(_)
                    | Message::View(_)
                    | Message::Action(_)
                    | Message::ActionResult(_)
                    | Message::MessageError(_)
            );
        }
    }
}
