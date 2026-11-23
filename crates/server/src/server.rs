mod game_match;
mod registration;

use game_match::Match;
use shared::errors::{GameError, GameResult};
use shared::maze::GeneratorAlgorithm;
use shared::messages::{
    Client, Message, MessageError, RegisterTeamResult, RegistrationError, SubscribePlayerResult,
    Teams, receive_message, send_message,
};
use std::collections::HashMap;
use std::fmt;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub seed: u64,
    pub max_players_per_team: u8,
    /// Maze size in cells.
    pub width: usize,
    pub height: usize,
    pub generator: GeneratorAlgorithm,
    /// How long after the first player joins the game starts even if teams are not full.
    pub start_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            seed: 0,
            max_players_per_team: 3,
            width: 50,
            height: 50,
            generator: GeneratorAlgorithm::RecursiveBacktracker,
            start_timeout: Duration::from_secs(30),
        }
    }
}

pub struct GameServer {
    clients: Arc<Mutex<HashMap<PlayerId, Client>>>,
    teams: Arc<Mutex<HashMap<String, Teams>>>,
    config: ServerConfig,
    /// The game, and the signal that wakes players waiting for it to start.
    game: Arc<(Mutex<Match>, Condvar)>,
}

/// Players are known by team and name, so two teams may use the same player names.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct PlayerId {
    team: String,
    name: String,
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.team, self.name)
    }
}

type ServerResult<T> = Result<T, RegistrationError>;

impl GameServer {
    pub fn new(config: ServerConfig) -> Self {
        let game = Arc::new((Mutex::new(Match::new(&config)), Condvar::new()));
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            teams: Arc::new(Mutex::new(HashMap::new())),
            config,
            game,
        }
    }

    pub fn run(&self) -> GameResult<()> {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&address).map_err(GameError::ConnectionError)?;

        info!("Server listening on {address}");
        self.serve(listener)
    }

    /// Accepts connections until the listener fails, one thread each.
    pub fn serve(&self, listener: TcpListener) -> GameResult<()> {
        for stream in listener.incoming() {
            let stream = stream.map_err(GameError::ConnectionError)?;
            let server = self.clone();
            std::thread::spawn(move || {
                if let Err(e) = server.handle_message(stream) {
                    error!("{e}");
                }
            });
        }
        Ok(())
    }

    fn has_started(&self) -> bool {
        self.lock_match().map_or(true, |game| game.started)
    }

    /// How many players have reached the exit so far.
    pub fn players_out(&self) -> usize {
        self.lock_match().map_or(0, |game| game.game.players().filter(|p| p.exited).count())
    }

    fn lock_match(&self) -> GameResult<MutexGuard<'_, Match>> {
        self.game.0.lock().map_err(|e| GameError::ThreadError(e.to_string()))
    }

    /// Puts the player on the board, and starts the game once every team is full.
    fn join_game(&self, player: &PlayerId) -> GameResult<()> {
        // Teams stay locked until the start decision, so no registration slips in between.
        let teams = self.teams.lock().map_err(|e| GameError::ThreadError(e.to_string()))?;
        let full = |team: &Teams| team.players.len() == self.config.max_players_per_team as usize;
        let all_full = !teams.is_empty() && teams.values().all(full);
        let registered: usize = teams.values().map(|team| team.players.len()).sum();

        let mut game = self.lock_match()?;
        // Only reachable when the start timeout fired while this player was joining.
        if game.started {
            game.add_player(player.clone());
            return Ok(());
        }
        game.first_join.get_or_insert_with(Instant::now);
        game.waiting.push(player.clone());
        // A teammate may be registered without having joined yet; starting before it does
        // would leave it off the board.
        if !game.started && all_full && game.waiting.len() == registered {
            game.start();
            info!("All teams are full, the game starts");
            self.game.1.notify_all();
        }
        Ok(())
    }

    /// Waits until the game starts: once every team is full, or once the start timeout
    /// has passed since the first player joined, with whoever joined by then.
    fn wait_for_start(&self, name: &PlayerId) -> GameResult<usize> {
        let mut game = self.lock_match()?;
        while !game.started {
            let deadline =
                game.first_join.map_or_else(Instant::now, |at| at + self.config.start_timeout);
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                game.start();
                info!("Start timeout reached, the game starts with the players who joined");
                self.game.1.notify_all();
                break;
            }
            game = self
                .game
                .1
                .wait_timeout(game, left)
                .map_err(|e| GameError::ThreadError(e.to_string()))?
                .0;
        }
        game.players
            .get(name)
            .copied()
            .ok_or_else(|| GameError::ThreadError(format!("{name} is not on the board")))
    }

    /// Runs a subscribed player's game until it reaches the exit or leaves.
    fn play(&self, stream: &mut TcpStream, name: &PlayerId) -> GameResult<()> {
        let outcome = self.play_turns(stream, name);
        if !matches!(outcome, Ok(true)) {
            let mut game = self.lock_match()?;
            if let Some(&id) = game.players.get(name) {
                game.game.leave(id);
            }
            info!("{name} left the game");
        }
        outcome.map(|_| ())
    }

    /// Plays turns; true once the player reaches the exit.
    fn play_turns(&self, stream: &mut TcpStream, name: &PlayerId) -> GameResult<bool> {
        let id = self.wait_for_start(name)?;
        send_message(stream, &self.lock_match()?.view(id))?;

        while let Ok(message) = receive_message(stream) {
            let Message::Action(action) = message else {
                warn!("{name} sent {message:?} during the game");
                let error =
                    MessageError { message: "Only actions are accepted during the game".into() };
                send_message(stream, &Message::MessageError(error))?;
                send_message(stream, &self.lock_match()?.view(id))?;
                continue;
            };
            let (replies, exited, moves) = {
                let mut game = self.lock_match()?;
                let (replies, exited) = game.act(id, action);
                if exited {
                    game.log_if_team_escaped(id);
                }
                (replies, exited, game.game.player(id).moves)
            };
            for reply in &replies {
                send_message(stream, reply)?;
            }
            if exited {
                info!("{name} found the exit in {moves} moves");
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn handle_message(&self, mut stream: TcpStream) -> GameResult<()> {
        let peer_address = stream.peer_addr().map_err(GameError::ConnectionError)?;

        loop {
            let message_result = receive_message(&mut stream);
            debug!("Received message from {peer_address}");

            match message_result {
                Ok(message) => {
                    let response = match message {
                        Message::RegisterTeam(team) => {
                            let team = Teams {
                                team_name: team.name,
                                registration_token: String::new(),
                                max_players: self.config.max_players_per_team,
                                players: Vec::new(),
                                score: 0,
                            };

                            match self.register_team(team) {
                                Ok(token) => Message::RegisterTeamResult(RegisterTeamResult::Ok {
                                    registration_token: token,
                                    expected_players: self.config.max_players_per_team,
                                }),
                                Err(err) => {
                                    error!("Team registration failed for {peer_address}: {err:?}");
                                    Message::RegisterTeamResult(RegisterTeamResult::Err(err))
                                }
                            }
                        }
                        Message::SubscribePlayer(player) => {
                            let player = Client {
                                player_name: player.name.clone(),
                                address: peer_address,
                                registration_token: player.registration_token,
                            };

                            // Joined before answering: a registered player that never joins
                            // would hold the start back forever.
                            let (result, id) = match self.register_client(player) {
                                Ok(id) => {
                                    self.join_game(&id)?;
                                    (SubscribePlayerResult::Ok, Some(id))
                                }
                                Err(err) => (SubscribePlayerResult::Err(err), None),
                            };
                            send_message(&mut stream, &Message::SubscribePlayerResult(result))?;
                            let Some(id) = id else {
                                continue;
                            };
                            // From here on this connection is the player's.
                            return self.play(&mut stream, &id);
                        }
                        _ => {
                            warn!("Received invalid message type from {peer_address}");
                            Message::MessageError(MessageError {
                                message: "Invalid message type".to_string(),
                            })
                        }
                    };

                    if let Err(e) = send_message(&mut stream, &response) {
                        error!("Failed to send message to {peer_address}: {e}");
                        break;
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }

        info!("Connection closed for {peer_address}");
        Ok(())
    }
}

impl Clone for GameServer {
    fn clone(&self) -> Self {
        Self {
            clients: Arc::clone(&self.clients),
            teams: Arc::clone(&self.teams),
            config: self.config.clone(),
            game: Arc::clone(&self.game),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::str::FromStr;

    fn setup_test_environment() -> GameServer {
        GameServer::new(ServerConfig::default())
    }

    fn create_test_client(name: &str, token: &str) -> Client {
        Client {
            player_name: name.to_string(),
            address: SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            registration_token: token.to_string(),
        }
    }

    fn create_test_team(name: &str) -> Teams {
        Teams {
            team_name: name.to_string(),
            players: Vec::new(),
            score: 0,
            registration_token: String::new(),
            max_players: 3,
        }
    }

    fn small_server(width: usize, height: usize, team_size: u8) -> GameServer {
        GameServer::new(ServerConfig {
            width,
            height,
            max_players_per_team: team_size,
            ..ServerConfig::default()
        })
    }

    #[test]
    fn test_subscription_refused_when_the_maze_has_no_free_cell() {
        // Two cells, one of them the exit: room for a single player.
        let server = small_server(1, 2, 2);
        let token = server.register_team(create_test_team("A")).unwrap();

        assert!(server.register_client(create_test_client("a", &token)).is_ok());
        assert!(matches!(
            server.register_client(create_test_client("b", &token)),
            Err(RegistrationError::TooManyPlayers)
        ));
    }

    #[test]
    fn test_game_starts_after_the_timeout_without_full_teams() {
        let server = GameServer::new(ServerConfig {
            width: 8,
            height: 8,
            max_players_per_team: 2,
            start_timeout: Duration::from_millis(100),
            ..ServerConfig::default()
        });
        let token = server.register_team(create_test_team("A")).unwrap();
        let player = create_test_client("a", &token);
        let name = server.register_client(player).unwrap();
        server.join_game(&name).unwrap();

        let (done, started) = std::sync::mpsc::channel();
        std::thread::spawn(move || done.send(server.wait_for_start(&name).is_ok()));

        assert_eq!(started.recv_timeout(Duration::from_secs(5)), Ok(true));
    }

    #[test]
    fn test_nobody_joins_once_the_game_started() {
        let server = small_server(8, 8, 1);
        let token = server.register_team(create_test_team("A")).unwrap();
        let player = create_test_client("a", &token);
        let id = server.register_client(player).unwrap();
        server.join_game(&id).unwrap();

        assert!(server.register_team(create_test_team("B")).is_err());
        assert!(server.register_client(create_test_client("a2", &token)).is_err());
    }

    #[test]
    fn test_register_team_success() {
        let server = setup_test_environment();
        let team = create_test_team("Test Team");
        let result = server.register_team(team);
        assert!(result.is_ok());

        let teams = server.teams.lock().unwrap();
        assert!(teams.contains_key("Test Team"));
    }

    #[test]
    fn test_register_team_duplicate() {
        let server = setup_test_environment();
        let team = create_test_team("Test Team");
        let team2 = create_test_team("Test Team");

        let _ = server.register_team(team);
        let result = server.register_team(team2);

        assert!(matches!(result, Err(RegistrationError::TeamAlreadyRegistered)));
    }

    #[test]
    fn test_generate_token() {
        let server = setup_test_environment();

        let token1 = server.generate_token();
        let token2 = server.generate_token();

        assert_ne!(token1, token2);
    }

    #[test]
    fn test_register_client_success() {
        let server = setup_test_environment();
        let team = create_test_team("Test Team");
        let token = server.register_team(team).unwrap();
        let client = create_test_client("Test Player", &token);

        let res = server.register_client(client);
        assert!(res.is_ok());

        let clients = server.clients.lock().unwrap();
        let id = PlayerId { team: "Test Team".into(), name: "Test Player".into() };
        assert!(clients.contains_key(&id));
    }

    #[test]
    fn test_players_on_different_teams_may_share_a_name() {
        let server = setup_test_environment();
        let alpha = server.register_team(create_test_team("Alpha")).unwrap();
        let beta = server.register_team(create_test_team("Beta")).unwrap();

        let first = server.register_client(create_test_client("Tremeaux1", &alpha));
        let second = server.register_client(create_test_client("Tremeaux1", &beta));

        assert!(first.is_ok());
        assert!(second.is_ok());
    }

    #[test]
    fn test_register_client_invalid_token() {
        let server = setup_test_environment();
        let client = create_test_client("Test Player", "invalid_token");

        let result = server.register_client(client);
        assert!(matches!(result, Err(RegistrationError::InvalidRegistrationToken)));
    }

    #[test]
    fn test_register_client_empty_name() {
        let server = setup_test_environment();
        let team = create_test_team("Test Team");
        let token = server.register_team(team).unwrap();
        let client = create_test_client("", &token);

        let result = server.register_client(client);
        assert!(matches!(result, Err(RegistrationError::InvalidName)));
    }

    #[test]
    fn test_register_multiple_clients_same_team() {
        let server = setup_test_environment();
        let team = create_test_team("Test Team");
        let token = server.register_team(team).unwrap();

        for i in 1..=3 {
            let client = create_test_client(&format!("Player{i}"), &token);
            let res = server.register_client(client);
            assert!(res.is_ok());
        }

        let teams = server.teams.lock().unwrap();
        let team = teams.get("Test Team").unwrap();
        assert_eq!(team.players.len(), 3);
    }

    #[test]
    fn test_register_client_concurrent() {
        use std::sync::Arc;
        use std::thread;

        let server = setup_test_environment();
        let server = Arc::new(server);
        let team = create_test_team("Test Team");
        let token = server.register_team(team).unwrap();

        let mut handles = vec![];
        for i in 0..3 {
            let server_clone = Arc::clone(&server);
            let token_clone = token.clone();

            handles.push(thread::spawn(move || {
                let client = create_test_client(&format!("Player{i}"), &token_clone);
                server_clone.register_client(client)
            }));
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let success_count = results.iter().filter(|&r| r.is_ok()).count();

        assert_eq!(success_count, 3);
    }
}
