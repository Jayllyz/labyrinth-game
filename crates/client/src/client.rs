use crate::instructions::{self, Algorithm};
use crate::maze_parser::maze_to_graph;
use crate::tui::GameState;
use crate::{data_structures::maze_graph::MazeGraph, maze_parser::Player};
use shared::utils::print_error;
use shared::{
    errors::{GameError, GameResult},
    logger::{LogLevel, Logger},
    maze::Cell,
    messages::{
        self, Action, Challenge, Hint, Message, RegisterTeam, RegisterTeamResult, SubscribePlayer,
        SubscribePlayerResult, receive_message, send_message,
    },
    radar::{decode_base64, extract_data},
};
use std::{
    collections::HashMap,
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
    thread::{self, ThreadId},
};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_addr: String,
    pub team_name: String,
}

pub struct GameClient {
    config: ClientConfig,
    secrets: TeamSecrets,
}

type TeamSecrets = Arc<Mutex<HashMap<ThreadId, u128>>>;

struct PlayerContext {
    graph: MazeGraph,
    player: Player,
    algorithm: Algorithm,
    challenge_modulo: Option<u128>,
    blocked_by_opponent: Option<Cell>,
}

struct LogContext {
    thread_name: String,
    tui_state: Option<Arc<Mutex<GameState>>>,
}

impl GameClient {
    pub fn new(config: ClientConfig) -> Self {
        Self { config, secrets: Arc::new(Mutex::new(HashMap::new())) }
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
                Err(GameError::TeamRegistrationError(format!("{:?}", err)))
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
        let mut handles = Vec::with_capacity(num_agents as usize);

        for i in 0..num_agents {
            let config = self.config.clone();
            let agent_token = token.clone();
            let agent_name = format!("Player{}", i + 1);
            let secrets = Arc::clone(&self.secrets);
            let tui_state = tui_state.clone();

            let handle = thread::Builder::new().name(agent_name.clone()).spawn(
                move || -> GameResult<()> {
                    let mut stream = Self::connect_to_server(&config.server_addr, max_retries)?;

                    let mut player_ctx = PlayerContext {
                        graph: MazeGraph::new(),
                        player: Player::new(),
                        algorithm,
                        challenge_modulo: None,
                        blocked_by_opponent: None,
                    };

                    let log_ctx = LogContext { thread_name: agent_name.clone(), tui_state };

                    send_message(
                        &mut stream,
                        &Message::SubscribePlayer(SubscribePlayer {
                            name: agent_name,
                            registration_token: agent_token,
                        }),
                    )?;

                    while let Ok(msg) = receive_message(&mut stream) {
                        Self::handle_server_message(
                            &mut stream,
                            msg,
                            &mut player_ctx,
                            &log_ctx,
                            &secrets,
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

    fn handle_server_message(
        stream: &mut impl Write,
        message: Message,
        player_ctx: &mut PlayerContext,
        log_ctx: &LogContext,
        secrets: &TeamSecrets,
    ) -> GameResult<()> {
        let logger = Logger::get_instance();

        if logger.is_debug_enabled() {
            Self::log_handler(
                &log_ctx.tui_state,
                &log_ctx.thread_name,
                logger,
                format!("Received message: {:?}", message),
                LogLevel::Debug,
            );
        }

        match message {
            Message::SubscribePlayerResult(result) => {
                Self::handle_subscription_result(result, log_ctx)?;
            }
            Message::RadarView(view) => {
                if Self::handle_radar_view(stream, view, player_ctx, log_ctx)? {
                    Self::log_handler(
                        &log_ctx.tui_state,
                        &log_ctx.thread_name,
                        logger,
                        "Found the exit!",
                        LogLevel::Info,
                    );
                    if let Some(tui) = &log_ctx.tui_state
                        && let Ok(mut state) = tui.lock()
                    {
                        state.mark_exit_found(&log_ctx.thread_name);
                    }
                    return Ok(());
                }
            }
            Message::Hint(hint) => {
                Self::handle_hint(hint, secrets);
            }
            Message::Challenge(value) => {
                Self::log_handler(
                    &log_ctx.tui_state,
                    &log_ctx.thread_name,
                    logger,
                    format!("{:?}", value),
                    LogLevel::Debug,
                );

                let Challenge::SecretSumModulo(modulo) = value;
                player_ctx.challenge_modulo = Some(modulo);
                Self::solve_challenge(stream, secrets, modulo)?;
            }
            Message::ActionError(err) => {
                let level = match err {
                    messages::ActionError::CannotPassThroughWall
                    | messages::ActionError::CannotPassThroughOpponent => LogLevel::Debug,
                    _ => LogLevel::Error,
                };
                Self::log_handler(
                    &log_ctx.tui_state,
                    &log_ctx.thread_name,
                    logger,
                    format!("Action error: {:?}", err),
                    level,
                );
                match err {
                    // The server follows these with a fresh RadarView, which drives the next move.
                    messages::ActionError::CannotPassThroughOpponent => {
                        let blocked = player_ctx.player.position;
                        player_ctx.player.revert_move();
                        player_ctx.blocked_by_opponent = Some(blocked);
                    }
                    messages::ActionError::CannotPassThroughWall
                    | messages::ActionError::InvalidMove
                    | messages::ActionError::OutOfMap
                    | messages::ActionError::Blocked => player_ctx.player.revert_move(),
                    messages::ActionError::NoRunningChallenge => {}
                    messages::ActionError::InvalidChallengeSolution
                    | messages::ActionError::SolveChallengeFirst => {
                        if let Some(modulo) = player_ctx.challenge_modulo {
                            Self::solve_challenge(stream, secrets, modulo)?;
                        }
                    }
                }
            }

            Message::MessageError(err) => {
                Self::log_handler(
                    &log_ctx.tui_state,
                    &log_ctx.thread_name,
                    logger,
                    format!("Server error: {:?}", err),
                    LogLevel::Error,
                );
            }
            _ => {
                Self::log_handler(
                    &log_ctx.tui_state,
                    &log_ctx.thread_name,
                    logger,
                    format!("Unhandled message: {:?}", message),
                    LogLevel::Warning,
                );
            }
        }

        Self::refresh_tui(
            &log_ctx.tui_state,
            &log_ctx.thread_name,
            &player_ctx.graph,
            &player_ctx.player,
        );

        Ok(())
    }

    fn handle_subscription_result(
        result: SubscribePlayerResult,
        log_ctx: &LogContext,
    ) -> GameResult<()> {
        let logger = Logger::get_instance();
        match result {
            SubscribePlayerResult::Ok => {
                Self::log_handler(
                    &log_ctx.tui_state,
                    &log_ctx.thread_name,
                    logger,
                    "Subscribed successfully",
                    LogLevel::Info,
                );
                Ok(())
            }
            SubscribePlayerResult::Err(err) => {
                Self::log_handler(
                    &log_ctx.tui_state,
                    &log_ctx.thread_name,
                    logger,
                    format!("Failed to subscribe: {:?}", err),
                    LogLevel::Error,
                );
                Err(GameError::AgentSubscriptionError(format!("{:?}", err)))
            }
        }
    }

    fn handle_radar_view(
        stream: &mut impl Write,
        view: messages::RadarView,
        player_ctx: &mut PlayerContext,
        log_ctx: &LogContext,
    ) -> GameResult<bool> {
        let radar_view = extract_data(&decode_base64(&view.0))
            .map_err(|e| GameError::MessageError(format!("Failed to decode radar view: {}", e)))?;

        maze_to_graph(&radar_view, &player_ctx.player, &mut player_ctx.graph);
        player_ctx.player.save_state();

        // Yield half the time so two agents blocking each other don't retry in lockstep.
        let blocked = player_ctx.blocked_by_opponent.take().filter(|_| rand::random_bool(0.5));
        let yielded = blocked.and_then(|cell| {
            instructions::yield_to_opponent(&mut player_ctx.player, &mut player_ctx.graph, cell)
        });

        let action: Action = match yielded {
            Some(action) => action,
            None => match player_ctx.algorithm {
                Algorithm::Tremeaux => {
                    instructions::tremeaux_solver(&mut player_ctx.player, &mut player_ctx.graph)
                }
                Algorithm::WallFollower => {
                    instructions::right_hand_solver(&radar_view, &mut player_ctx.player)
                }
                Algorithm::Alian => instructions::alian_solver(
                    &mut player_ctx.player,
                    &mut player_ctx.graph,
                    &log_ctx.thread_name,
                ),
            },
        };

        send_message(stream, &Message::Action(action.clone()))?;

        Ok(instructions::check_win_condition(&radar_view.cells, action))
    }

    fn handle_hint(hint: Hint, secrets: &TeamSecrets) {
        if let Hint::Secret(secret) = hint
            && let Ok(mut secrets) = secrets.lock()
        {
            secrets.insert(thread::current().id(), secret);
        }
    }

    fn solve_challenge(
        stream: &mut impl Write,
        secrets: &TeamSecrets,
        modulo: u128,
    ) -> GameResult<()> {
        let answer = match secrets.lock() {
            Ok(secrets) => instructions::solve_sum_modulo(modulo, &secrets),
            Err(_) => return Ok(()),
        };
        send_message(stream, &Message::Action(Action::SolveChallenge { answer }))
    }

    fn refresh_tui(
        tui_state: &Option<Arc<Mutex<GameState>>>,
        thread_name: &str,
        graph: &MazeGraph,
        player: &Player,
    ) {
        if let Some(tui) = tui_state
            && let Ok(mut state) = tui.lock()
        {
            state.update_state(thread_name, graph.clone(), player.clone());
        }
    }

    fn log_handler(
        tui_state: &Option<Arc<Mutex<GameState>>>,
        thread_name: &str,
        logger: &Logger,
        message: impl Into<String>,
        level: LogLevel,
    ) {
        if matches!(level, LogLevel::Debug) && !logger.is_debug_enabled() {
            return;
        }

        match tui_state.as_ref() {
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

    fn create_test_contexts(thread_name: String) -> (PlayerContext, LogContext, TeamSecrets) {
        Logger::init(true);
        let game_ctx = PlayerContext {
            graph: MazeGraph::new(),
            player: Player::new(),
            algorithm: Algorithm::Tremeaux,
            challenge_modulo: None,
            blocked_by_opponent: None,
        };

        let log_ctx = LogContext { thread_name, tui_state: None };

        (game_ctx, log_ctx, Arc::new(Mutex::new(HashMap::new())))
    }

    fn sent_messages(buf: Vec<u8>) -> Vec<Message> {
        let mut cursor = std::io::Cursor::new(buf);
        let mut messages = Vec::new();
        while (cursor.position() as usize) < cursor.get_ref().len() {
            messages.push(receive_message(&mut cursor).unwrap());
        }
        messages
    }

    fn answers(buf: Vec<u8>) -> Vec<String> {
        sent_messages(buf)
            .into_iter()
            .map(|msg| match msg {
                Message::Action(Action::SolveChallenge { answer }) => answer,
                other => panic!("unexpected message: {:?}", other),
            })
            .collect()
    }

    #[test]
    fn test_failed_move_waits_for_next_radar_view() {
        for err in [
            messages::ActionError::CannotPassThroughWall,
            messages::ActionError::CannotPassThroughOpponent,
            messages::ActionError::NoRunningChallenge,
        ] {
            let (mut game_ctx, log_ctx, secrets) = create_test_contexts("Player1".to_string());
            let mut sent = Vec::new();

            GameClient::handle_server_message(
                &mut sent,
                Message::ActionError(err),
                &mut game_ctx,
                &log_ctx,
                &secrets,
            )
            .unwrap();

            assert!(sent.is_empty());
        }
    }

    #[test]
    fn test_opponent_block_reverts_and_remembers_blocked_cell() {
        let (mut game_ctx, log_ctx, secrets) = create_test_contexts("Player1".to_string());
        game_ctx.player.save_state();
        game_ctx.player.move_forward();
        let blocked = game_ctx.player.position;

        GameClient::handle_server_message(
            &mut Vec::new(),
            Message::ActionError(messages::ActionError::CannotPassThroughOpponent),
            &mut game_ctx,
            &log_ctx,
            &secrets,
        )
        .unwrap();

        assert_eq!(game_ctx.player.position, Player::new().position);
        assert_eq!(game_ctx.blocked_by_opponent, Some(blocked));
    }

    #[test]
    fn test_each_agent_answers_challenge_with_its_own_modulo() {
        let (mut agent_a, log_a, secrets) = create_test_contexts("Player1".to_string());
        let (mut agent_b, log_b, _) = create_test_contexts("Player2".to_string());
        secrets.lock().unwrap().insert(thread::current().id(), 25);

        let mut sent = Vec::new();
        let challenge = |m| Message::Challenge(Challenge::SecretSumModulo(m));
        let invalid = Message::ActionError(messages::ActionError::InvalidChallengeSolution);
        GameClient::handle_server_message(&mut sent, challenge(10), &mut agent_a, &log_a, &secrets)
            .unwrap();
        GameClient::handle_server_message(&mut sent, challenge(7), &mut agent_b, &log_b, &secrets)
            .unwrap();
        GameClient::handle_server_message(&mut sent, invalid, &mut agent_a, &log_a, &secrets)
            .unwrap();

        assert_eq!(answers(sent), ["5", "4", "5"]);
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
        let client =
            GameClient::new(ClientConfig { server_addr: addr, team_name: "team".to_string() });

        let token = client.register_team(&mut stream).unwrap();
        assert_eq!(token, "test_token");
    }

    #[test]
    fn test_handle_server_message_subscribe_success() {
        let (listener, addr) = setup_mock_server();

        thread::spawn(move || {
            listener.accept().unwrap();
        });

        let mut stream = TcpStream::connect(addr).unwrap();
        let message = Message::SubscribePlayerResult(SubscribePlayerResult::Ok);
        let (mut game_ctx, log_ctx, secrets) = create_test_contexts("Player1".to_string());

        let result = GameClient::handle_server_message(
            &mut stream,
            message,
            &mut game_ctx,
            &log_ctx,
            &secrets,
        );

        assert!(result.is_ok());
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
    fn test_handle_radar_view() {
        let (listener, addr) = setup_mock_server();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let msg = receive_message(&mut stream).unwrap();
            assert!(matches!(msg, Message::Action(_)));
        });

        let mut stream = TcpStream::connect(addr).unwrap();
        let message = Message::RadarView(messages::RadarView("bKgGjsIyap8p8aa".to_string()));
        let (mut game_ctx, log_ctx, secrets) = create_test_contexts("Player1".to_string());

        let result = GameClient::handle_server_message(
            &mut stream,
            message,
            &mut game_ctx,
            &log_ctx,
            &secrets,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_hint() {
        let (listener, addr) = setup_mock_server();
        thread::spawn(move || if let Ok((_stream, _)) = listener.accept() {});

        let mut stream = TcpStream::connect(addr).unwrap();
        let (mut game_ctx, log_ctx, secrets) = create_test_contexts("Player1".to_string());

        let message = Message::Hint(Hint::Secret(10));

        let result = GameClient::handle_server_message(
            &mut stream,
            message,
            &mut game_ctx,
            &log_ctx,
            &secrets,
        );

        assert!(result.is_ok());
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

        let config = ClientConfig { server_addr: addr, team_name: "team".to_string() };
        let client = GameClient::new(config);

        let result = client.run(1, 1, None, Algorithm::Tremeaux);
        assert!(result.is_ok());
    }
}
