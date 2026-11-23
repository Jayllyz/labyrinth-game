use crate::game::{Game, Moved, Position};
use rand::{RngExt, SeedableRng, rng, rngs::StdRng};
use shared::errors::{GameError, GameResult};
use shared::heading::Heading;
use shared::maze::{GeneratorAlgorithm, Maze};
use shared::messages::{
    Action, ActionError, Challenge, Client, Direction, Hint, Message, MessageError, RadarView,
    RegisterTeamResult, RegistrationError, SubscribePlayerResult, Teams, receive_message,
    send_message,
};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
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
    clients: Arc<Mutex<HashMap<String, Client>>>,
    teams: Arc<Mutex<HashMap<String, Teams>>>,
    config: ServerConfig,
    /// The game, and the signal that wakes players waiting for it to start.
    game: Arc<(Mutex<Match>, Condvar)>,
}

/// Odds per move of each random message, close to what the reference server sends.
const SECRET_ODDS: f64 = 0.012;
const COMPASS_ODDS: f64 = 0.012;
const GRID_SIZE_ODDS: f64 = 0.001;
const CHALLENGE_ODDS: f64 = 0.001;

type ServerResult<T> = Result<T, RegistrationError>;

/// One game: the board, and which player and team each name stands for.
struct Match {
    game: Game,
    rng: StdRng,
    players: HashMap<String, usize>,
    team_names: Vec<String>,
    /// Players who joined, with their team, until the game starts and places them.
    waiting: Vec<(String, String)>,
    first_join: Option<Instant>,
    started: bool,
}

impl Match {
    /// Builds the maze and places the exit from the seed, so a game can be replayed.
    fn new(config: &ServerConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(config.seed);
        let maze =
            Maze::generate(config.generator, config.width, config.height, false, config.seed);
        let exit = random_cell(&mut rng, config);
        Self {
            game: Game::new(maze, exit),
            rng,
            players: HashMap::new(),
            team_names: Vec::new(),
            waiting: Vec::new(),
            first_join: None,
            started: false,
        }
    }

    /// Places a new player on a free cell, facing North like on the reference server.
    /// Places everyone in name order, so a seed gives each player the same cell whatever
    /// order they connected in.
    fn start(&mut self) {
        let mut waiting = std::mem::take(&mut self.waiting);
        waiting.sort();
        for (name, team) in waiting {
            self.add_player(&name, &team);
        }
        self.started = true;
    }

    fn add_player(&mut self, name: &str, team: &str) -> usize {
        let team = self.team_names.iter().position(|t| t == team).unwrap_or_else(|| {
            self.team_names.push(team.to_string());
            self.team_names.len() - 1
        });
        // Subscriptions stop once the maze is full, so a free cell is always left.
        let free = self.game.free_cells();
        let position = free[self.rng.random_range(0..free.len())];
        let id = self.game.add_player(team, position, Heading::North);
        self.players.insert(name.to_string(), id);
        id
    }

    fn view(&self, id: usize) -> Message {
        Message::RadarView(RadarView(self.game.radar(id)))
    }

    /// The messages that answer a player's action, and whether it reached the exit.
    fn act(&mut self, id: usize, action: Action) -> (Vec<Message>, bool) {
        match action {
            Action::MoveTo(direction) => self.move_player(id, direction),
            Action::SolveChallenge { answer } => (self.solve(id, &answer), false),
        }
    }

    fn move_player(&mut self, id: usize, direction: Direction) -> (Vec<Message>, bool) {
        if self.game.player(id).challenge.is_some() {
            return (vec![Message::ActionError(ActionError::SolveChallengeFirst)], false);
        }
        match self.game.apply_move(id, direction) {
            Ok(Moved::Stepped) => (self.after_move(id), false),
            // The reference server shows the exit under the player once more before closing.
            Ok(Moved::Exited) => (vec![self.view(id)], true),
            Err(err) => (vec![Message::ActionError(err), self.view(id)], false),
        }
    }

    /// The optional hint, then the view or a new challenge that replaces it.
    fn after_move(&mut self, id: usize) -> Vec<Message> {
        let mut replies = Vec::new();
        if let Some(hint) = self.hint(id) {
            replies.push(Message::Hint(hint));
        }
        if self.rng.random_bool(CHALLENGE_ODDS) {
            let modulo = self.rng.random_range(2..=1000);
            self.game.player_mut(id).challenge = Some(modulo);
            replies.push(Message::Challenge(Challenge::SecretSumModulo(modulo)));
        } else {
            replies.push(self.view(id));
        }
        replies
    }

    fn hint(&mut self, id: usize) -> Option<Hint> {
        if self.rng.random_bool(GRID_SIZE_ODDS) {
            let (columns, rows) = self.game.size();
            return Some(Hint::GridSize { columns, rows });
        }
        if self.rng.random_bool(SECRET_ODDS) {
            let secret = self.rng.random::<u64>().into();
            self.game.player_mut(id).secret = Some(secret);
            return Some(Hint::Secret(secret));
        }
        self.rng
            .random_bool(COMPASS_ODDS)
            .then(|| Hint::RelativeCompass { angle: self.game.exit_bearing(id) })
    }

    fn solve(&mut self, id: usize, answer: &str) -> Vec<Message> {
        let Some(modulo) = self.game.player(id).challenge else {
            return vec![Message::ActionError(ActionError::NoRunningChallenge), self.view(id)];
        };
        self.game.count_action(id);
        let expected = self.game.team_secret_sum(self.game.player(id).team, modulo);
        if answer.trim().parse::<u128>().ok() != Some(expected) {
            return vec![Message::ActionError(ActionError::InvalidChallengeSolution)];
        }
        self.game.player_mut(id).challenge = None;
        vec![self.view(id)]
    }

    /// Logs the team's score once its last member is out.
    fn log_if_team_escaped(&self, id: usize) {
        let team = self.game.player(id).team;
        let members: Vec<_> = self.game.players().filter(|p| p.team == team).collect();
        if members.iter().all(|p| p.exited) {
            let moves: u32 = members.iter().map(|p| p.moves).sum();
            let score = f64::from(moves) / members.len() as f64;
            info!("Team {} escaped with a score of {score:.1}", self.team_names[team]);
        }
    }
}

fn random_cell(rng: &mut StdRng, config: &ServerConfig) -> Position {
    Position {
        x: rng.random_range(0..config.width) as i16,
        y: rng.random_range(0..config.height) as i16,
    }
}

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
    fn join_game(&self, player: &Client) -> GameResult<()> {
        // Teams stay locked until the start decision, so no registration slips in between.
        let teams = self.teams.lock().map_err(|e| GameError::ThreadError(e.to_string()))?;
        let team = self.find_team_by_token(&teams, &player.registration_token).unwrap_or_default();
        let full = |team: &Teams| team.players.len() == self.config.max_players_per_team as usize;
        let all_full = !teams.is_empty() && teams.values().all(full);
        let registered: usize = teams.values().map(|team| team.players.len()).sum();

        let mut game = self.lock_match()?;
        // Only reachable when the start timeout fired while this player was joining.
        if game.started {
            game.add_player(&player.player_name, &team);
            return Ok(());
        }
        game.first_join.get_or_insert_with(Instant::now);
        game.waiting.push((player.player_name.clone(), team));
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
    fn wait_for_start(&self, name: &str) -> GameResult<usize> {
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
    fn play(&self, stream: &mut TcpStream, name: &str) -> GameResult<()> {
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
    fn play_turns(&self, stream: &mut TcpStream, name: &str) -> GameResult<bool> {
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

    fn register_client(&self, player: Client) -> SubscribePlayerResult {
        if player.player_name.is_empty() {
            return SubscribePlayerResult::Err(RegistrationError::InvalidName);
        }

        if player.registration_token.is_empty() {
            return SubscribePlayerResult::Err(RegistrationError::InvalidRegistrationToken);
        }

        let Ok(mut clients) = self.clients.lock() else {
            return SubscribePlayerResult::Err(RegistrationError::ServerError);
        };

        let Ok(mut teams) = self.teams.lock() else {
            return SubscribePlayerResult::Err(RegistrationError::ServerError);
        };

        if clients.contains_key(&player.player_name) {
            return SubscribePlayerResult::Err(RegistrationError::AlreadyRegistered);
        }

        // Every player needs a start cell other than the exit, and the board is set once
        // the game starts.
        if clients.len() + 1 >= self.config.width * self.config.height || self.has_started() {
            return SubscribePlayerResult::Err(RegistrationError::TooManyPlayers);
        }

        let Some(team_name) = self.find_team_by_token(&teams, &player.registration_token) else {
            return SubscribePlayerResult::Err(RegistrationError::InvalidRegistrationToken);
        };

        if let Some(team) = teams.get_mut(&team_name) {
            if team.players.len() >= self.config.max_players_per_team as usize {
                return SubscribePlayerResult::Err(RegistrationError::TooManyPlayers);
            }

            team.players.push(player.clone());
            clients.insert(player.player_name.clone(), player.clone());

            info!("{} registered successfully on team {}", player.player_name, team_name);
            SubscribePlayerResult::Ok
        } else {
            error!("Team {team_name} not found");
            SubscribePlayerResult::Err(RegistrationError::ServerError)
        }
    }

    fn find_team_by_token(&self, teams: &HashMap<String, Teams>, token: &str) -> Option<String> {
        teams
            .iter()
            .find(|(_, team)| team.registration_token == token)
            .map(|(name, _)| name.clone())
    }

    fn register_team(&self, mut team: Teams) -> ServerResult<String> {
        let mut teams = self.teams.lock().map_err(|_| RegistrationError::ServerError)?;

        if teams.contains_key(&team.team_name) {
            return Err(RegistrationError::TeamAlreadyRegistered);
        }
        if self.has_started() {
            return Err(RegistrationError::TooManyPlayers);
        }

        let token = self.generate_token();
        let team_name = team.team_name.clone();
        team.registration_token.clone_from(&token);

        teams.insert(team_name.clone(), team);

        info!("Team {team_name} registered successfully");
        Ok(token)
    }

    fn generate_token(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get timestamp")
            .as_secs()
            .to_string();
        let random_part: String =
            rng().sample_iter(&rand::distr::Alphanumeric).take(16).map(char::from).collect();

        format!("{timestamp}{random_part}")
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
                                team_name: String::new(),
                                address: peer_address,
                                registration_token: player.registration_token,
                            };

                            let name = player.player_name.clone();
                            let result = self.register_client(player.clone());
                            let joined = matches!(result, SubscribePlayerResult::Ok);
                            // Joined before answering: a registered player that never joins
                            // would hold the start back forever.
                            if joined {
                                self.join_game(&player)?;
                            }
                            send_message(&mut stream, &Message::SubscribePlayerResult(result))?;
                            if !joined {
                                continue;
                            }
                            // From here on this connection is the player's.
                            return self.play(&mut stream, &name);
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
            team_name: "Test Team".to_string(),
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

        assert!(matches!(
            server.register_client(create_test_client("a", &token)),
            SubscribePlayerResult::Ok
        ));
        assert!(matches!(
            server.register_client(create_test_client("b", &token)),
            SubscribePlayerResult::Err(RegistrationError::TooManyPlayers)
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
        server.register_client(player.clone());
        server.join_game(&player).unwrap();

        let (done, started) = std::sync::mpsc::channel();
        std::thread::spawn(move || done.send(server.wait_for_start("a").is_ok()));

        assert_eq!(started.recv_timeout(Duration::from_secs(5)), Ok(true));
    }

    #[test]
    fn test_nobody_joins_once_the_game_started() {
        let server = small_server(8, 8, 1);
        let token = server.register_team(create_test_team("A")).unwrap();
        let player = create_test_client("a", &token);
        server.register_client(player.clone());
        server.join_game(&player).unwrap();

        assert!(server.register_team(create_test_team("B")).is_err());
        assert!(matches!(
            server.register_client(create_test_client("a2", &token)),
            SubscribePlayerResult::Err(_)
        ));
    }

    /// A started game with two teammates, the first one in a challenge.
    fn game_in_challenge(modulo: u128) -> Match {
        let mut game = Match::new(&ServerConfig { width: 8, height: 8, ..ServerConfig::default() });
        game.waiting = vec![("a".into(), "A".into()), ("b".into(), "A".into())];
        game.start();
        game.game.player_mut(0).challenge = Some(modulo);
        game
    }

    #[test]
    fn test_challenge_must_be_solved_before_moving() {
        let mut game = game_in_challenge(10);
        game.game.player_mut(0).secret = Some(11);
        game.game.player_mut(1).secret = Some(32);

        let (replies, _) = game.act(0, Action::MoveTo(Direction::Front));
        assert!(matches!(replies[..], [Message::ActionError(ActionError::SolveChallengeFirst)]));

        let (replies, _) = game.act(0, Action::SolveChallenge { answer: "4".into() });
        assert!(matches!(
            replies[..],
            [Message::ActionError(ActionError::InvalidChallengeSolution)]
        ));

        let (replies, _) = game.act(0, Action::SolveChallenge { answer: "3".into() });
        assert!(matches!(replies.last(), Some(Message::RadarView(_))));
        assert_eq!(game.game.player_mut(0).challenge, None);
    }

    #[test]
    fn test_teammate_without_secret_counts_as_zero() {
        let mut game = game_in_challenge(7);
        game.game.player_mut(0).secret = Some(9);

        let (replies, _) = game.act(0, Action::SolveChallenge { answer: "2".into() });

        assert!(matches!(replies.last(), Some(Message::RadarView(_))));
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
        assert!(matches!(res, SubscribePlayerResult::Ok));

        let clients = server.clients.lock().unwrap();
        assert!(clients.contains_key("Test Player"));
    }

    #[test]
    fn test_register_client_invalid_token() {
        let server = setup_test_environment();
        let client = create_test_client("Test Player", "invalid_token");

        let result = server.register_client(client);
        assert!(matches!(
            result,
            SubscribePlayerResult::Err(RegistrationError::InvalidRegistrationToken)
        ));
    }

    #[test]
    fn test_register_client_empty_name() {
        let server = setup_test_environment();
        let team = create_test_team("Test Team");
        let token = server.register_team(team).unwrap();
        let client = create_test_client("", &token);

        let result = server.register_client(client);
        assert!(matches!(result, SubscribePlayerResult::Err(RegistrationError::InvalidName)));
    }

    #[test]
    fn test_register_multiple_clients_same_team() {
        let server = setup_test_environment();
        let team = create_test_team("Test Team");
        let token = server.register_team(team).unwrap();

        for i in 1..=3 {
            let client = create_test_client(&format!("Player{i}"), &token);
            let res = server.register_client(client);
            assert!(matches!(res, SubscribePlayerResult::Ok));
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
        let success_count =
            results.iter().filter(|&r| matches!(r, SubscribePlayerResult::Ok)).count();

        assert_eq!(success_count, 3);
    }
}
