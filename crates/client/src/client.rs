use shared::messages::{receive_message, Message, SubscribePlayerResult};
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_addr: String,
    pub player_name: String,
    pub team_name: String,
}

pub struct GameClient {
    config: ClientConfig,
    #[allow(dead_code)]
    score: u32,
}

impl GameClient {
    pub fn new(config: ClientConfig) -> Self {
        Self { config, score: 0 }
    }

    pub fn run(&self, max_retries: u8) {
        let mut stream = Self::connect_to_server(&self.config.server_addr, max_retries);

        while let Ok(message) = receive_message(&mut stream) {
            Self::handle_server_message(self.config.clone(), &mut stream, message);
        }
    }

    fn connect_to_server(address: &str, mut max_retries: u8) -> TcpStream {
        loop {
            match TcpStream::connect(address) {
                Ok(stream) => return stream,
                Err(e) => {
                    eprintln!("Failed to connect to server: {}", e);
                    max_retries -= 1;
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    if max_retries == 0 {
                        eprintln!("Max retries reached, exiting...");
                        std::process::exit(1);
                    }
                    continue;
                }
            }
        }
    }

    fn handle_server_message(_config: ClientConfig, _stream: &mut TcpStream, message: Message) {
        match message {
            Message::SubscribePlayerResult(result) => match result {
                SubscribePlayerResult::Ok => {
                    eprintln!("Subscribed successfully");
                    std::process::exit(1);
                }
                SubscribePlayerResult::Err(err) => {
                    eprintln!("Subscribe error: {:?}", err);
                }
            },
            Message::MessageError(err) => {
                eprintln!("Error: {}", err.message);
            }
            _ => {
                eprintln!("Unexpected message, exiting...");
                std::process::exit(1);
            }
        }
    }
}
