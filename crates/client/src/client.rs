use shared::{
    messages::{
        receive_message, send_message, Message, RegisterTeam, RegisterTeamResult, SubscribePlayer,
        SubscribePlayerResult,
    },
    utils::{print_error, print_log, Color},
};
use std::{error::Error, io, net::TcpStream};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_addr: String,
    pub player_name: String,
    pub team_name: String,
    pub token: Option<String>,
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

    pub fn run(&self, max_retries: u8) -> Result<(), Box<dyn Error>> {
        let mut stream = Self::connect_to_server(&self.config.server_addr, max_retries)?;

        let init_message = if let Some(token) = &self.config.token {
            Message::SubscribePlayer(SubscribePlayer {
                name: self.config.player_name.clone(),
                registration_token: token.clone(),
            })
        } else {
            Message::RegisterTeam(RegisterTeam { name: self.config.team_name.clone() })
        };

        send_message(&mut stream, &init_message);

        loop {
            match receive_message(&mut stream) {
                Ok(message) => {
                    Self::handle_server_message(self.config.clone(), &mut stream, message)?;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }

    fn connect_to_server(address: &str, mut max_retries: u8) -> io::Result<TcpStream> {
        loop {
            match TcpStream::connect(address) {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    print_error(&format!("Failed to connect to server: {}", e));
                    if max_retries == 0 {
                        print_error("Max retries reached, exiting...");
                        return Err(e);
                    }
                    max_retries -= 1;
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }
    }

    fn handle_server_message(
        config: ClientConfig,
        _stream: &mut TcpStream,
        message: Message,
    ) -> Result<(), Box<dyn Error>> {
        match message {
            Message::RegisterTeamResult(result) => match result {
                RegisterTeamResult::Ok { registration_token, .. } => {
                    print_log(
                        &format!(
                            "Team {} registered successfully, token: {}",
                            config.team_name, registration_token
                        ),
                        Color::Reset,
                    );
                    std::process::exit(0);
                }
                RegisterTeamResult::Err(err) => {
                    print_error(&format!("Failed to register team: {:?}", err));
                    std::process::exit(1);
                }
            },
            Message::SubscribePlayerResult(result) => match result {
                SubscribePlayerResult::Ok => {
                    print_log("Successfully subscribed to game", Color::Green);
                }
                SubscribePlayerResult::Err(err) => {
                    print_error(&format!("Subscribe error: {:?}", err));
                    std::process::exit(1);
                }
            },
            Message::MessageError(err) => {
                print_error(&format!("Server error: {}", err.message));
                std::process::exit(1);
            }
            _ => {
                print_error("Unexpected message from server");
                std::process::exit(1);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    fn setup_mock_server() -> (TcpListener, String) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        (listener, format!("127.0.0.1:{}", addr.port()))
    }

    #[test]
    fn test_connect_to_server() {
        let (listener, addr) = setup_mock_server();

        thread::spawn(move || {
            listener.accept().unwrap();
        });

        let result = GameClient::connect_to_server(&addr, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_server_message_register_success() {
        let config = ClientConfig {
            server_addr: "".to_string(),
            player_name: "player".to_string(),
            team_name: "team".to_string(),
            token: None,
        };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let mut stream = TcpStream::connect(addr).unwrap();

        let message = Message::RegisterTeamResult(RegisterTeamResult::Ok {
            expected_players: 1,
            registration_token: "token123".to_string(),
        });

        let result = GameClient::handle_server_message(config, &mut stream, message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_server_message_subscribe_success() {
        let config = ClientConfig {
            server_addr: "".to_string(),
            player_name: "player".to_string(),
            team_name: "team".to_string(),
            token: Some("token".to_string()),
        };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let mut stream = TcpStream::connect(addr).unwrap();

        let message = Message::SubscribePlayerResult(SubscribePlayerResult::Ok);

        let result = GameClient::handle_server_message(config, &mut stream, message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_client() {
        let config = ClientConfig {
            server_addr: "addr".to_string(),
            player_name: "player".to_string(),
            team_name: "team".to_string(),
            token: None,
        };

        let client = GameClient::new(config.clone());
        assert_eq!(client.config.server_addr, config.server_addr);
        assert_eq!(client.config.player_name, config.player_name);
        assert_eq!(client.config.team_name, config.team_name);
        assert_eq!(client.score, 0);
    }
}
