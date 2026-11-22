use crate::agent::{Agent, TeamSecrets};
use crate::instructions::Algorithm;
use crate::tui::GameState;
use shared::utils::print_error;
use shared::{
    errors::{GameError, GameResult},
    logger::{LogLevel, Logger},
    messages::{
        Action, Message, RegisterTeam, RegisterTeamResult, SubscribePlayer, receive_message,
        send_message,
    },
};
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_addr: String,
    pub team_name: String,
    /// Pause before each move, so a game can be followed in the TUI.
    pub move_delay: Duration,
}

pub struct GameClient {
    config: ClientConfig,
    secrets: TeamSecrets,
}

/// Name of the agent at `index`, shared by the server subscription and the TUI tabs.
pub fn agent_name(algorithm: Algorithm, index: u8) -> String {
    format!("{:?}{}", algorithm, index + 1)
}

impl GameClient {
    pub fn new(config: ClientConfig) -> Self {
        Self { config, secrets: TeamSecrets::default() }
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

    fn register_team(&self, stream: &mut TcpStream) -> GameResult<String> {
        send_message(
            stream,
            &Message::RegisterTeam(RegisterTeam { name: self.config.team_name.clone() }),
        )?;

        match receive_message(stream)? {
            Message::RegisterTeamResult(RegisterTeamResult::Ok { registration_token, .. }) => {
                Ok(registration_token)
            }
            Message::RegisterTeamResult(RegisterTeamResult::Err(err)) => {
                Err(GameError::TeamRegistrationError(format!("{err:?}")))
            }
            _ => Err(GameError::MessageError("Invalid registration response".into())),
        }
    }

    pub fn run(
        &self,
        max_retries: u8,
        num_agents: u8,
        tui_state: Option<Arc<Mutex<GameState>>>,
        algorithm: Algorithm,
    ) -> GameResult<()> {
        let mut stream = Self::connect_to_server(&self.config.server_addr, max_retries)?;
        let token = self.register_team(&mut stream)?;
        let (config, token) = (&self.config, &token);

        thread::scope(|scope| {
            let handles = (0..num_agents)
                .map(|i| {
                    let name = agent_name(algorithm, i);
                    let mut agent = Agent::new(name.clone(), algorithm, Arc::clone(&self.secrets));
                    let tui_state = tui_state.clone();
                    thread::Builder::new().name(name.clone()).spawn_scoped(
                        scope,
                        move || -> GameResult<()> {
                            let mut stream =
                                Self::connect_to_server(&config.server_addr, max_retries)?;
                            send_message(
                                &mut stream,
                                &Message::SubscribePlayer(SubscribePlayer {
                                    name,
                                    registration_token: token.clone(),
                                }),
                            )?;
                            Self::play(&mut stream, &mut agent, &tui_state, config.move_delay)
                        },
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

            handles.into_iter().try_for_each(|handle| {
                handle.join().map_err(|e| GameError::ThreadError(format!("{e:?}")))?
            })
        })
    }

    /// Feeds server messages to the agent and sends its actions back, until the server
    /// closes the connection, which it does once the agent reaches the exit. Guessing the
    /// win from the radar misses exits other players stand on.
    fn play(
        stream: &mut (impl Read + Write),
        agent: &mut Agent,
        tui_state: &Option<Arc<Mutex<GameState>>>,
        move_delay: Duration,
    ) -> GameResult<()> {
        let logger = Logger::get_instance();
        while let Ok(message) = receive_message(stream) {
            if logger.is_debug_enabled() {
                Self::log(
                    tui_state,
                    agent.name(),
                    format!("Received message: {message:?}"),
                    LogLevel::Debug,
                );
            }

            let reply = agent.on_message(message).inspect_err(|err| {
                Self::log(tui_state, agent.name(), err.to_string(), LogLevel::Error);
            })?;
            if let Some((level, text)) = reply.log {
                Self::log(tui_state, agent.name(), text, level);
            }
            Self::refresh_tui(tui_state, agent);

            let Some(action) = reply.action else {
                continue;
            };
            if matches!(action, Action::MoveTo(_)) {
                thread::sleep(move_delay);
            }
            send_message(stream, &Message::Action(action))?;
        }

        Self::log(tui_state, agent.name(), "Found the exit!", LogLevel::Info);
        if let Some(tui) = tui_state
            && let Ok(mut state) = tui.lock()
        {
            state.mark_exit_found(agent.name());
        }
        Ok(())
    }

    fn refresh_tui(tui_state: &Option<Arc<Mutex<GameState>>>, agent: &Agent) {
        if let Some(tui) = tui_state
            && let Ok(mut state) = tui.lock()
        {
            state.update_state(agent.name(), agent.graph().clone(), agent.player().clone());
        }
    }

    fn log(
        tui_state: &Option<Arc<Mutex<GameState>>>,
        agent_name: &str,
        message: impl Into<String>,
        level: LogLevel,
    ) {
        let logger = Logger::get_instance();
        if matches!(level, LogLevel::Debug) && !logger.is_debug_enabled() {
            return;
        }

        match tui_state.as_ref() {
            Some(tui) => {
                if let Ok(mut state) = tui.lock() {
                    state.add_log(agent_name, message.into(), level);
                }
            }
            None => match level {
                LogLevel::Debug => logger.debug(&format!("{} {}", agent_name, message.into())),
                LogLevel::Info => logger.info(&format!("{} {}", agent_name, message.into())),
                LogLevel::Error => logger.error(&format!("{} {}", agent_name, message.into())),
                LogLevel::Warning => logger.warn(&format!("{} {}", agent_name, message.into())),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::messages::{Challenge, Hint, RadarView, SubscribePlayerResult};
    use std::net::{TcpListener, TcpStream};

    fn setup_mock_server() -> (TcpListener, String) {
        Logger::init(true);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        (listener, format!("127.0.0.1:{}", addr.port()))
    }

    #[test]
    fn test_agent_found_the_exit_when_server_closes_connection() {
        Logger::init(true);
        let mut agent = Agent::new("Player1".into(), Algorithm::Tremeaux, TeamSecrets::default());
        let tui = Arc::new(Mutex::new(GameState::new()));
        tui.lock().unwrap().register_agent("Player1".to_string());
        let mut closed_stream = std::io::Cursor::new(Vec::new());

        GameClient::play(&mut closed_stream, &mut agent, &Some(Arc::clone(&tui)), Duration::ZERO)
            .unwrap();

        assert!(tui.lock().unwrap().has_found_exit("Player1"));
    }

    #[test]
    fn test_agents_are_named_after_their_algorithm() {
        assert_eq!(agent_name(Algorithm::Alian, 0), "Alian1");
        assert_eq!(agent_name(Algorithm::WallFollower, 2), "WallFollower3");
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
    fn test_register_team() {
        let (listener, addr) = setup_mock_server();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
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
        });

        let mut stream = TcpStream::connect(addr.clone()).unwrap();
        let client = GameClient::new(ClientConfig {
            server_addr: addr,
            team_name: "team".to_string(),
            move_delay: Duration::ZERO,
        });

        let token = client.register_team(&mut stream).unwrap();
        assert_eq!(token, "test_token");
    }

    #[test]
    fn test_new_client() {
        let config = ClientConfig {
            server_addr: "addr".to_string(),
            team_name: "team".to_string(),
            move_delay: Duration::ZERO,
        };

        let client = GameClient::new(config.clone());
        assert_eq!(client.config.server_addr, config.server_addr);
        assert_eq!(client.config.team_name, config.team_name);
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
                        &Message::RadarView(RadarView("bKgGjsIyap8p8aa".to_string())),
                    )
                    .unwrap();

                    let msg = receive_message(&mut player_stream).unwrap();
                    assert!(matches!(msg, Message::Action(_)));
                }
            }
        });

        let config = ClientConfig {
            server_addr: addr,
            team_name: "team".to_string(),
            move_delay: Duration::ZERO,
        };
        let client = GameClient::new(config);

        let result = client.run(1, 1, None, Algorithm::Tremeaux);
        assert!(result.is_ok());
    }
}
