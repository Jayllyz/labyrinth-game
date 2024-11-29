use shared::{
    messages::{
        receive_message, send_message, Message, RegisterTeam, RegisterTeamResult, SubscribePlayer,
        SubscribePlayerResult,
    },
    radar::{decode_base64, extract_data},
    utils::{print_error, print_log, Color},
};
use std::{error::Error, net::TcpStream, sync::mpsc, thread};

use crate::instructions;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_addr: String,
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

    pub fn run(&self, max_retries: u8, num_agents: u8) -> Result<(), Box<dyn Error>> {
        let (token_tx, token_rx) = mpsc::channel();

        let mut stream = Self::connect_to_server(&self.config.server_addr, max_retries);
        let register_msg =
            Message::RegisterTeam(RegisterTeam { name: self.config.team_name.clone() });

        send_message(&mut stream, &register_msg)?;

        match receive_message(&mut stream)? {
            Message::RegisterTeamResult(RegisterTeamResult::Ok { registration_token, .. }) => {
                token_tx.send(registration_token)?;
            }
            _ => return Err("Failed to register team".into()),
        }

        let mut handles = vec![];
        let token = match token_rx.recv() {
            Ok(token) => token,
            Err(e) => return Err(format!("Failed to receive token: {:?}", e).into()),
        };

        for i in 0..num_agents {
            let config = self.config.clone();
            let agent_token = token.clone();
            let agent_name = format!("Player{}", i + 1);

            let handle = thread::Builder::new()
                .name(agent_name.clone())
                .spawn(move || {
                    let mut stream = Self::connect_to_server(&config.server_addr, max_retries);
                    let subscribe_msg = Message::SubscribePlayer(SubscribePlayer {
                        name: agent_name,
                        registration_token: agent_token,
                    });

                    match send_message(&mut stream, &subscribe_msg) {
                        Ok(_) => {}
                        Err(e) => {
                            print_error(&format!("Failed to subscribe player: {}", e));
                        }
                    }

                    while let Ok(msg) = receive_message(&mut stream) {
                        Self::handle_server_message(&mut stream, msg);
                    }
                })
                .map_err(|e| format!("Failed to spawn thread: {}", e))?;
            handles.push(handle);
        }

        for handle in handles {
            handle.join().map_err(|e| format!("Thread panicked: {:?}", e))?;
        }
        Ok(())
    }

    fn connect_to_server(address: &str, mut max_retries: u8) -> TcpStream {
        loop {
            match TcpStream::connect(address) {
                Ok(stream) => return stream,
                Err(e) => {
                    print_error(&format!("Failed to connect to server: {}", e));
                    if max_retries == 1 {
                        println!("Max retries reached, exiting...");
                        std::process::exit(1);
                    }
                    max_retries -= 1;
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }
    }

    fn handle_server_message(stream: &mut TcpStream, message: Message) {
        match message {
            Message::SubscribePlayerResult(result) => match result {
                SubscribePlayerResult::Ok => {
                    print_log("Successfully subscribed to game", Color::Green);
                }
                SubscribePlayerResult::Err(err) => {
                    print_error(&format!("Subscribe error: {:?}", err));
                    std::process::exit(1);
                }
            },
            Message::RadarView(view) => {
                let (horizontal, vertical, cells) = extract_data(&decode_base64(&view.0));
                let action = instructions::right_hand_solver(horizontal, vertical);
                let is_win = instructions::check_win_condition(cells, action.clone());
                match send_message(stream, &Message::Action(action)) {
                    Ok(_) => {}
                    Err(e) => {
                        print_log(
                            &format!("[warning] - Failed to send message: {}", e),
                            Color::Orange,
                        );
                    }
                }

                if is_win {
                    let thread = std::thread::current();
                    if let Some(name) = thread.name() {
                        print_log(
                            &format!("Thread {} has won the game with score {}", name, 0),
                            Color::Green,
                        );
                    }
                    thread.unpark();
                }
            }
            Message::Hint(_hint) => {
                print_log("Hint received", Color::Green);
            }
            Message::MessageError(err) => {
                print_error(&format!("Server error: {}", err.message));
                std::process::exit(1);
            }
            _ => {
                print_error("Unexpected message from server");
                std::process::exit(1);
            }
        }
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

        let result = std::panic::catch_unwind(|| GameClient::connect_to_server(&addr, 1));
        assert!(result.is_ok());

        let register_msg = Message::RegisterTeam(RegisterTeam { name: "team".to_string() });

        let mut stream = TcpStream::connect(addr).unwrap();
        send_message(&mut stream, &register_msg).unwrap();
    }

    #[test]
    fn test_handle_server_message_subscribe_success() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let mut stream = TcpStream::connect(addr).unwrap();

        let message = Message::SubscribePlayerResult(SubscribePlayerResult::Ok);

        GameClient::handle_server_message(&mut stream, message);
    }

    #[test]
    fn test_new_client() {
        let config = ClientConfig {
            server_addr: "addr".to_string(),
            team_name: "team".to_string(),
            token: None,
        };

        let client = GameClient::new(config.clone());
        assert_eq!(client.config.server_addr, config.server_addr);
        assert_eq!(client.config.team_name, config.team_name);
        assert_eq!(client.score, 0);
    }
}
