use crate::instructions;
use crate::maze_parser::maze_to_graph;
use crate::{data_structures::maze_graph::MazeGraph, maze_parser::Player};
use shared::maze::Cell;
use shared::messages::Direction;
use shared::{
    logger::Logger,
    messages::{
        self, receive_message, send_message, Action, Challenge, Hint, Message, RegisterTeam,
        RegisterTeamResult, SubscribePlayer, SubscribePlayerResult,
    },
    radar::{decode_base64, extract_data},
    utils::print_error,
};
use std::{
    collections::HashMap,
    error::Error,
    net::TcpStream,
    sync::{Arc, Mutex},
    thread::{self, ThreadId},
};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_addr: String,
    pub team_name: String,
    pub token: Option<String>,
}

struct SecretSumModulo {
    sum: Arc<Mutex<u128>>,
    secrets: Arc<Mutex<HashMap<ThreadId, u128>>>,
}

pub struct GameClient {
    config: ClientConfig,
    challenge_secret_sum: SecretSumModulo,
}

impl GameClient {
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            challenge_secret_sum: SecretSumModulo {
                sum: Arc::new(Mutex::new(0)),
                secrets: Arc::new(Mutex::new(HashMap::new())),
            },
        }
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

    pub fn run(&self, max_retries: u8, num_agents: u8) -> Result<(), Box<dyn Error>> {
        let logger = Logger::get_instance();
        let mut stream = Self::connect_to_server(&self.config.server_addr, max_retries);
        let register_msg =
            Message::RegisterTeam(RegisterTeam { name: self.config.team_name.clone() });
        send_message(&mut stream, &register_msg)?;

        let token = match receive_message(&mut stream)? {
            Message::RegisterTeamResult(RegisterTeamResult::Ok { registration_token, .. }) => {
                registration_token
            }
            _ => return Err("Failed to register team".into()),
        };

        let mut handles = vec![];
        for i in 0..num_agents {
            let config = self.config.clone();
            let agent_token = token.clone();
            let agent_name = format!("Player{}", i + 1);
            let secrets_sum = SecretSumModulo {
                sum: Arc::clone(&self.challenge_secret_sum.sum),
                secrets: Arc::clone(&self.challenge_secret_sum.secrets),
            };

            let mut graph = MazeGraph::new();
            let mut player =
                Player { position: Cell { row: 0, column: 0 }, direction: Direction::Front };

            let handle = thread::Builder::new()
                .name(agent_name.clone())
                .spawn(move || {
                    let mut stream = Self::connect_to_server(&config.server_addr, max_retries);
                    let subscribe_msg = Message::SubscribePlayer(SubscribePlayer {
                        name: agent_name.clone(),
                        registration_token: agent_token,
                    });
                    match send_message(&mut stream, &subscribe_msg) {
                        Ok(_) => {}
                        Err(e) => {
                            logger.error(&format!("Failed to send message: {}", e));
                        }
                    }
                    while let Ok(msg) = receive_message(&mut stream) {
                        Self::handle_server_message(
                            &mut stream,
                            &agent_name,
                            msg,
                            &secrets_sum,
                            &mut graph,
                            &mut player,
                        );
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

    fn handle_server_message(
        stream: &mut TcpStream,
        thread_name: &str,
        message: Message,
        secrets_sum: &SecretSumModulo,
        graph: &mut MazeGraph,
        player: &mut Player,
    ) {
        let logger = Logger::get_instance();
        let thread = std::thread::current();

        logger.debug(&format!("{} received message: {:?}", thread_name, message));

        match message {
            Message::SubscribePlayerResult(result) => match result {
                SubscribePlayerResult::Ok => {
                    logger.info(&format!("{} has successfully subscribed to game", thread_name));
                }
                SubscribePlayerResult::Err(err) => {
                    logger.error(&format!("{} failed to subscribe: {:?}", thread_name, err));
                }
            },
            Message::RadarView(view) => {
                Self::handle_radar_view(stream, logger, thread_name, view, graph, player);
            }
            Message::Hint(hint) => {
                if let Hint::Secret(secret) = hint {
                    if let Ok(mut secrets) = secrets_sum.secrets.lock() {
                        secrets.insert(thread.id(), secret);
                    }
                }
            }
            Message::Challenge(value) => {
                logger.info(&format!("{} received challenge: {:?}", thread_name, value));

                match value {
                    Challenge::SecretSumModulo(challenge) => {
                        Self::handle_secret_sum_modulo(
                            stream,
                            &secrets_sum.secrets,
                            &secrets_sum.sum,
                            Some(challenge),
                        );
                    }
                }
            }
            Message::ActionError(err) => match err {
                messages::ActionError::InvalidChallengeSolution => {
                    logger.info(&format!("{} failed to solve challenge, retrying...", thread_name));
                    Self::handle_secret_sum_modulo(
                        stream,
                        &secrets_sum.secrets,
                        &secrets_sum.sum,
                        None,
                    );
                }
                messages::ActionError::InvalidMove => todo!(),
                messages::ActionError::OutOfMap => todo!(),
                messages::ActionError::Blocked => todo!(),
                messages::ActionError::SolveChallengeFirst => todo!(),
            },
            Message::MessageError(err) => {
                logger.error(&format!("Server error: {}", err.message));
            }
            _ => {
                logger.warn(&format!("Unhandled message: {:?}", message));
            }
        }
    }

    fn handle_secret_sum_modulo(
        stream: &mut TcpStream,
        secrets: &Arc<Mutex<HashMap<ThreadId, u128>>>,
        secret_sum: &Arc<Mutex<u128>>,
        new_sum: Option<u128>,
    ) {
        if let Ok(mut sum) = secret_sum.lock() {
            if let Some(new_sum) = new_sum {
                *sum = new_sum;
            }
            if let Ok(secrets) = secrets.lock() {
                let result = instructions::solve_sum_modulo(*sum, &secrets);
                let _ = send_message(
                    stream,
                    &Message::Action(Action::SolveChallenge { answer: result }),
                );
            }
        }
    }

    fn handle_radar_view(
        stream: &mut TcpStream,
        logger: &Logger,
        thread_name: &str,
        view: messages::RadarView,
        graph: &mut MazeGraph,
        player: &mut Player,
    ) {
        let radar_view = extract_data(&decode_base64(&view.0));
        maze_to_graph(&radar_view, player, graph);
        let action = instructions::right_hand_solver(&radar_view, player);
        if let Err(e) = send_message(stream, &Message::Action(action.clone())) {
            logger.error(&format!("{} failed to send action: {}", thread_name, e));
        }

        let is_win = instructions::check_win_condition(&radar_view.cells, action);
        if is_win {
            logger.info(&format!("{} has found the exit!", thread_name));
            print!("{:?}", graph);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    fn setup_mock_server() -> (TcpListener, String) {
        Logger::init(true);
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

        let register_msg = Message::RegisterTeam(RegisterTeam { name: "team".to_string() });

        let mut stream = TcpStream::connect(addr).unwrap();
        send_message(&mut stream, &register_msg).unwrap();
    }

    #[test]
    fn test_handle_server_message_subscribe_success() {
        let (listener, addr) = setup_mock_server();

        thread::spawn(move || {
            listener.accept().unwrap();
        });
        let mut stream = TcpStream::connect(addr).unwrap();

        let message = Message::SubscribePlayerResult(SubscribePlayerResult::Ok);
        let mut graph = MazeGraph::new();
        let mut player =
            Player { position: Cell { row: 0, column: 0 }, direction: Direction::Front };

        GameClient::handle_server_message(
            &mut stream,
            "Player1",
            message,
            &SecretSumModulo {
                sum: Arc::new(Mutex::new(0)),
                secrets: Arc::new(Mutex::new(HashMap::new())),
            },
            &mut graph,
            &mut player,
        );
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
    }

    #[test]
    fn test_handle_secret_sum_modulo() {
        let (listener, addr) = setup_mock_server();

        thread::spawn(move || {
            listener.accept().unwrap();
        });

        let mut stream = TcpStream::connect(addr).unwrap();

        let secrets = Arc::new(Mutex::new(HashMap::new()));

        let secret_sum = Arc::new(Mutex::new(0));

        let message = Message::Challenge(messages::Challenge::SecretSumModulo(10));

        let mut graph = MazeGraph::new();
        let mut player =
            Player { position: Cell { row: 0, column: 0 }, direction: Direction::Front };

        GameClient::handle_server_message(
            &mut stream,
            "Player1",
            message,
            &SecretSumModulo { sum: secret_sum, secrets },
            &mut graph,
            &mut player,
        );
    }

    #[test]
    fn test_handle_radar_view() {
        let (listener, addr) = setup_mock_server();

        thread::spawn(move || {
            listener.accept().unwrap();
        });

        let mut stream = TcpStream::connect(addr).unwrap();

        let message = Message::RadarView(messages::RadarView("bKgGjsIyap8p8aa".to_string()));

        let mut graph = MazeGraph::new();
        let mut player =
            Player { position: Cell { row: 0, column: 0 }, direction: Direction::Front };

        GameClient::handle_server_message(
            &mut stream,
            "Player1",
            message,
            &SecretSumModulo {
                sum: Arc::new(Mutex::new(0)),
                secrets: Arc::new(Mutex::new(HashMap::new())),
            },
            &mut graph,
            &mut player,
        );
    }
}
