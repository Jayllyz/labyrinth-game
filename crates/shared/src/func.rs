use std::{
    env,
    io::{BufReader, Read, Write},
    net::TcpStream,
};

use crate::messages::Message;

const MAX_REQUEST_SIZE: u64 = 32 * 1024;

pub fn get_server_address() -> String {
    const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:7878";
    env::args()
        .nth(1)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string())
}

pub fn receive_message(stream: &mut TcpStream) -> Result<Message, String> {
    let buf_reader = BufReader::with_capacity(8192, stream); // 8KB buffer
    let mut buffer = String::new();

    match buf_reader.take(MAX_REQUEST_SIZE).read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => eprintln!("Error reading from connection: {}", e),
    }

    if buffer.len() >= MAX_REQUEST_SIZE as usize {
        eprintln!("Request too large");
        return Err("Request too large".to_string());
    }

    let request = String::from_utf8_lossy(buffer.as_bytes());
    println!("\n\x1b[34mReceived: {}\x1b[0m", request);

    match serde_json::from_str(&request) {
        Ok(msg) => Ok(msg),
        Err(e) => Err(format!("Error parsing message: {}", e)),
    }
}

pub fn send_message(stream: &mut TcpStream, msg: Message) {
    let serialized = serde_json::to_string(&msg).unwrap();
    println!("\n\x1b[32mSending: {}\x1b[0m", serialized);

    stream.write_all(serialized.as_bytes()).expect("Failed to write to stream");
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
