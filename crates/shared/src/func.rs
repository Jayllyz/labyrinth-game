use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
};

use crate::messages::Message;

pub fn get_server_address() -> String {
    const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:7878";
    env::args()
        .nth(1)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string())
}

pub fn get_player_name() -> String {
    const DEFAULT_PLAYER_NAME: &str = "Player1";
    env::args().nth(2).filter(|s| !s.is_empty()).unwrap_or_else(|| DEFAULT_PLAYER_NAME.to_string())
}

pub fn receive_message(stream: &mut TcpStream) -> Result<Message, String> {
    let mut buf_len = [0u8; 4];
    match stream.read_exact(&mut buf_len) {
        Ok(_) => (),
        Err(e) => Err(format!("Failed to read message length: {}", e))?,
    }

    let len = u32::from_be_bytes(buf_len) as usize;
    if len > 1_000_000 {
        Err(format!("Message too large: {} bytes", len))?
    }

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

    println!("\n\x1b[34mReceived : {}\x1b[0m", serde_json::to_string_pretty(&json).unwrap());

    Ok(json)
}

pub fn send_message(stream: &mut TcpStream, msg: Message) {
    let json = serde_json::to_string(&msg).expect("Failed to serialize message");
    println!("\n\x1b[32mSending: {}\x1b[0m", json);

    let len = json.len() as u32;
    stream.write_all(&len.to_be_bytes()).expect("Failed to write to stream");
    stream.write_all(json.as_bytes()).expect("Failed to write to stream");
    stream.flush().expect("Failed to flush stream");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_server_address() {
        let address = get_server_address();
        const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:7878";
        assert_eq!(address, DEFAULT_SERVER_ADDRESS);
    }
}
