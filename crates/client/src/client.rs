use crate::instructions;
use crate::maze_parser::maze_to_graph;
use crate::tui::GameState;
use crate::{data_structures::maze_graph::MazeGraph, maze_parser::Player};
use shared::utils::print_error;
use shared::{
    errors::{GameError, GameResult},
    logger::{LogLevel, Logger},
    messages::{
        self, receive_message, send_message, Action, Challenge, Hint, Message, RegisterTeam,
        RegisterTeamResult, SubscribePlayer, SubscribePlayerResult,
    },
    radar::{decode_base64, extract_data},
};
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{Arc, Mutex},
    thread::{self, ThreadId},
};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_addr: String,
    pub team_name: String,
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

    fn connect_to_server(address: &str, mut max_retries: u8) -> GameResult<TcpStream> {
        loop {
            match TcpStream::connect(address) {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    print_error(&e.to_string());
                    if max_retries == 1 {
                        return Err(GameError::ConnectionError(e));
                    }
                    max_retries -= 1;
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }
    }

    pub fn run(
        &self,
        max_retries: u8,
        num_agents: u8,
        tui_state: Option<Arc<Mutex<GameState>>>,
        algorithm: String,
    ) -> GameResult<()> {
        let mut stream = Self::connect_to_server(&self.config.server_addr, max_retries)?;

        send_message(
            &mut stream,
            &Message::RegisterTeam(RegisterTeam { name: self.config.team_name.clone() }),
        )?;

        let token = match receive_message(&mut stream)? {
            Message::RegisterTeamResult(RegisterTeamResult::Ok { registration_token, .. }) => {
                registration_token
            }
            Message::RegisterTeamResult(RegisterTeamResult::Err(err)) => {
                return Err(GameError::TeamRegistrationError(format!("{:?}", err)));
            }
            _ => return Err(GameError::MessageError("Invalid registration response".into())),
        };

        let mut handles = Vec::with_capacity(num_agents as usize);

        for i in 0..num_agents {
            let config = self.config.clone();
            let agent_token = token.clone();
            let agent_name = format!("Player{}", i + 1);
            let secrets_sum = SecretSumModulo {
                sum: Arc::clone(&self.challenge_secret_sum.sum),
                secrets: Arc::clone(&self.challenge_secret_sum.secrets),
            };
            let tui_state = tui_state.clone();
            let algorithm = algorithm.clone();

            let handle = thread::Builder::new().name(agent_name.clone()).spawn(
                move || -> GameResult<()> {
                    let mut stream = Self::connect_to_server(&config.server_addr, max_retries)?;
                    let mut graph = MazeGraph::new();
                    let mut player = Player::new();

                    send_message(
                        &mut stream,
                        &Message::SubscribePlayer(SubscribePlayer {
                            name: agent_name.clone(),
                            registration_token: agent_token,
                        }),
                    )?;

                    while let Ok(msg) = receive_message(&mut stream) {
                        Self::handle_server_message(
                            &mut stream,
                            &agent_name,
                            msg,
                            &secrets_sum,
                            &mut graph,
                            &mut player,
                            tui_state.as_ref(),
                            &algorithm,
                        )?;
                    }
                    Ok(())
                },
            )?;

            handles.push(handle);
        }

        handles
            .into_iter()
            .map(|handle| handle.join().map_err(|e| GameError::ThreadError(format!("{:?}", e)))?)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_server_message(
        stream: &mut TcpStream,
        thread_name: &str,
        message: Message,
        secrets_sum: &SecretSumModulo,
        graph: &mut MazeGraph,
        player: &mut Player,
        tui_state: Option<&Arc<Mutex<GameState>>>,
        algorithm: &str,
    ) -> GameResult<()> {
        let logger = Logger::get_instance();
        let thread = std::thread::current();

        if logger.is_debug_enabled() {
            Self::log_handler(
                tui_state,
                thread_name,
                logger,
                format!("Received message: {:?}", message),
                LogLevel::Debug,
            );
        }

        match message {
            Message::SubscribePlayerResult(result) => match result {
                SubscribePlayerResult::Ok => {
                    Self::log_handler(
                        tui_state,
                        thread_name,
                        logger,
                        "Subscribed successfully",
                        LogLevel::Info,
                    );
                }
                SubscribePlayerResult::Err(err) => {
                    Self::log_handler(
                        tui_state,
                        thread_name,
                        logger,
                        format!("Failed to subscribe: {:?}", err),
                        LogLevel::Error,
                    );
                    return Err(GameError::AgentSubscriptionError(format!("{:?}", err)));
                }
            },
            Message::RadarView(view) => {
                if Self::handle_radar_view(stream, view, graph, player, thread_name, algorithm)? {
                    Self::log_handler(
                        tui_state,
                        thread_name,
                        logger,
                        "Found the exit!",
                        LogLevel::Info,
                    );
                    return Ok(());
                }
            }
            Message::Hint(hint) => {
                if let Hint::Secret(secret) = hint {
                    if let Ok(mut secrets) = secrets_sum.secrets.lock() {
                        secrets.insert(thread.id(), secret);
                    }
                }
            }
            Message::Challenge(value) => {
                Self::log_handler(
                    tui_state,
                    thread_name,
                    logger,
                    format!("{:?}", value),
                    LogLevel::Info,
                );

                match value {
                    Challenge::SecretSumModulo(challenge) => {
                        Self::handle_secret_sum_modulo(
                            stream,
                            &secrets_sum.secrets,
                            &secrets_sum.sum,
                            Some(challenge),
                        )?;
                    }
                }
            }
            Message::ActionError(err) => match err {
                messages::ActionError::InvalidChallengeSolution => {
                    Self::log_handler(
                        tui_state,
                        thread_name,
                        logger,
                        "Invalid challenge solution, retrying...",
                        LogLevel::Error,
                    );
                    Self::handle_secret_sum_modulo(
                        stream,
                        &secrets_sum.secrets,
                        &secrets_sum.sum,
                        None,
                    )?;
                }
                messages::ActionError::InvalidMove => todo!(),
                messages::ActionError::OutOfMap => todo!(),
                messages::ActionError::Blocked => todo!(),
                messages::ActionError::SolveChallengeFirst => todo!(),
            },
            Message::MessageError(err) => {
                Self::log_handler(
                    tui_state,
                    thread_name,
                    logger,
                    format!("Server error: {:?}", err),
                    LogLevel::Error,
                );
            }
            _ => {
                Self::log_handler(
                    tui_state,
                    thread_name,
                    logger,
                    format!("Unhandled message: {:?}", message),
                    LogLevel::Warning,
                );
            }
        }

        Self::refresh_tui(tui_state, thread_name, graph, player);

        Ok(())
    }

    fn handle_secret_sum_modulo(
        stream: &mut TcpStream,
        secrets: &Arc<Mutex<HashMap<ThreadId, u128>>>,
        secret_sum: &Arc<Mutex<u128>>,
        new_sum: Option<u128>,
    ) -> GameResult<()> {
        if let Ok(mut sum) = secret_sum.lock() {
            if let Some(new_sum) = new_sum {
                *sum = new_sum;
            }
            if let Ok(secrets) = secrets.lock() {
                let result = instructions::solve_sum_modulo(*sum, &secrets);
                send_message(stream, &Message::Action(Action::SolveChallenge { answer: result }))?;
            }
        }
        Ok(())
    }

    fn handle_radar_view(
        stream: &mut TcpStream,
        view: messages::RadarView,
        graph: &mut MazeGraph,
        player: &mut Player,
        thread_name: &str,
        algorithm: &str,
    ) -> GameResult<bool> {
        let radar_view = extract_data(&decode_base64(&view.0))
            .map_err(|e| GameError::MessageError(format!("Failed to decode radar view: {}", e)))?;

        maze_to_graph(&radar_view, player, graph);

        let action: Action = match algorithm {
            "Tremeaux" => instructions::tremeaux_solver(player, graph),
            "RightHand" => instructions::right_hand_solver(&radar_view, player),
            "Alian" => instructions::alian_solver(player, graph, thread_name),
            _ => instructions::tremeaux_solver(player, graph),
        };

        // let action = instructions::alian_solver(player, graph, thread_name);
        send_message(stream, &Message::Action(action.clone()))?;

        let is_win = instructions::check_win_condition(&radar_view.cells, action);
        if is_win {
            return Ok(true);
        }

        Ok(false)
    }

    fn refresh_tui(
        tui_state: Option<&Arc<Mutex<GameState>>>,
        thread_name: &str,
        graph: &MazeGraph,
        player: &Player,
    ) {
        if let Some(tui) = tui_state {
            if let Ok(mut state) = tui.lock() {
                state.update_state(thread_name, graph.clone(), player.clone());
            }
        }
    }

    fn log_handler(
        tui_state: Option<&Arc<Mutex<GameState>>>,
        thread_name: &str,
        logger: &Logger,
        message: impl Into<String>,
        level: LogLevel,
    ) {
        match tui_state {
            Some(tui) => {
                if let Ok(mut state) = tui.lock() {
                    state.add_log(thread_name, message.into(), level)
                }
            }
            None => match level {
                LogLevel::Debug => logger.debug(&format!("{} {}", thread_name, message.into())),
                LogLevel::Info => logger.info(&format!("{} {}", thread_name, message.into())),
                LogLevel::Error => logger.error(&format!("{} {}", thread_name, message.into())),
                LogLevel::Warning => logger.warn(&format!("{} {}", thread_name, message.into())),
            },
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
        let mut player = Player::new();

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
            None,
            "Tremeaux",
        )
        .unwrap();
    }

    #[test]
    fn test_new_client() {
        let config =
            ClientConfig { server_addr: "addr".to_string(), team_name: "team".to_string() };

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
        let mut player = Player::new();

        GameClient::handle_server_message(
            &mut stream,
            "Player1",
            message,
            &SecretSumModulo { sum: secret_sum, secrets },
            &mut graph,
            &mut player,
            None,
            "Tremeaux",
        )
        .unwrap();
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
        let mut player = Player::new();

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
            None,
            "Tremeaux",
        )
        .unwrap();
    }

    #[test]
    fn test_handle_hint() {
        let (listener, addr) = setup_mock_server();

        thread::spawn(move || {
            listener.accept().unwrap();
        });

        let mut stream = TcpStream::connect(addr).unwrap();

        let message = Message::Hint(Hint::Secret(10));

        let mut graph = MazeGraph::new();
        let mut player = Player::new();

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
            None,
            "Tremeaux",
        )
        .unwrap();
    }

    #[test]
    fn test_run_function() {
        let (listener, addr) = setup_mock_server();

        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let msg = receive_message(&mut stream).unwrap();
                assert!(matches!(msg, Message::RegisterTeam(_)));

                send_message(
                    &mut stream,
                    &Message::RegisterTeamResult(RegisterTeamResult::Ok {
                        registration_token: "test_token".to_string(),
                        expected_players: 1,
                    }),
                )
                .unwrap();

                if let Ok((mut player_stream, _)) = listener.accept() {
                    let msg = receive_message(&mut player_stream).unwrap();
                    assert!(matches!(msg, Message::SubscribePlayer(_)));

                    send_message(
                        &mut player_stream,
                        &Message::SubscribePlayerResult(SubscribePlayerResult::Ok),
                    )
                    .unwrap();

                    send_message(&mut player_stream, &Message::Hint(Hint::Secret(42))).unwrap();

                    send_message(
                        &mut player_stream,
                        &Message::Challenge(Challenge::SecretSumModulo(100)),
                    )
                    .unwrap();

                    let msg = receive_message(&mut player_stream).unwrap();
                    assert!(matches!(msg, Message::Action(Action::SolveChallenge { .. })));

                    send_message(
                        &mut player_stream,
                        &Message::RadarView(messages::RadarView("bKgGjsIyap8p8aa".to_string())),
                    )
                    .unwrap();

                    let msg = receive_message(&mut player_stream).unwrap();
                    assert!(matches!(msg, Message::Action(_)));
                }
            }
        });

        let config = ClientConfig { server_addr: addr.clone(), team_name: "team".to_string() };
        let client = GameClient::new(config);

        client.run(1, 1, None, "Tremeaux".to_owned()).unwrap();
    }
}
