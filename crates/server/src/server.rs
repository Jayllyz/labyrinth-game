use crate::game::{Game, Moved, Position};
use rand::{RngExt, SeedableRng, rng, rngs::StdRng};
use shared::errors::{GameError, GameResult};
use shared::heading::Heading;
use shared::maze_generator::sidewinder;
use shared::messages::{
    Action, ActionError, Client, Message, MessageError, RadarView, RegisterTeamResult,
    RegistrationError, SubscribePlayerResult, Teams, receive_message, send_message,
};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};
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

type ServerResult<T> = Result<T, RegistrationError>;

/// One game: the board, and which player and team each name stands for.
struct Match {
    game: Game,
    rng: StdRng,
    players: HashMap<String, usize>,
    team_names: Vec<String>,
    /// Players who joined, with their team, until the game starts and places them.
    waiting: Vec<(String, String)>,
    started: bool,
}

impl Match {
    /// Builds the maze and places the exit from the seed, so a game can be replayed.
    fn new(config: &ServerConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(config.seed);
        let maze = sidewinder(config.width, config.height, false, config.seed);
        let exit = random_cell(&mut rng, config);
        Self {
            game: Game::new(maze, exit),
            rng,
            players: HashMap::new(),
            team_names: Vec::new(),
            waiting: Vec::new(),
            started: false,
        }
    }

    /// Places a new player on a free cell, facing North like on the reference server.
    /// Places everyone in name order, so a seed gives each player the same cell whatever
    /// order they connected in.
    fn start(&mut self, config: &ServerConfig) {
        let mut waiting = std::mem::take(&mut self.waiting);
        waiting.sort();
        for (name, team) in waiting {
            self.add_player(&name, &team, config);
        }
        self.started = true;
    }

    fn add_player(&mut self, name: &str, team: &str, config: &ServerConfig) -> usize {
        let team = self.team_names.iter().position(|t| t == team).unwrap_or_else(|| {
            self.team_names.push(team.to_string());
            self.team_names.len() - 1
        });
        let position = loop {
            let cell = random_cell(&mut self.rng, config);
            if self.game.is_free(cell) {
                break cell;
            }
        };
        let id = self.game.add_player(team, position, Heading::North);
        self.players.insert(name.to_string(), id);
        id
    }

    fn view(&self, id: usize) -> Message {
        Message::RadarView(RadarView(self.game.radar(id)))
    }

    /// The messages that answer a player's action, and whether it reached the exit.
    fn act(&mut self, id: usize, action: Action) -> (Vec<Message>, bool) {
        let Action::MoveTo(direction) = action else {
            return (
                vec![Message::ActionError(ActionError::NoRunningChallenge), self.view(id)],
                false,
            );
        };
        match self.game.apply_move(id, direction) {
            Ok(Moved::Stepped) => (vec![self.view(id)], false),
            // The reference server shows the exit under the player once more before closing.
            Ok(Moved::Exited) => (vec![self.view(id)], true),
            Err(err) => (vec![Message::ActionError(err), self.view(id)], false),
        }
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

    /// How many players have reached the exit so far.
    pub fn players_out(&self) -> usize {
        self.lock_match().map_or(0, |game| game.game.players().filter(|p| p.exited).count())
    }

    fn lock_match(&self) -> GameResult<MutexGuard<'_, Match>> {
        self.game.0.lock().map_err(|e| GameError::ThreadError(e.to_string()))
    }

    /// Puts the player on the board, and starts the game once every team is full.
    fn join_game(&self, player: &Client) -> GameResult<()> {
        let teams = self.teams.lock().map_err(|e| GameError::ThreadError(e.to_string()))?;
        let team = self.find_team_by_token(&teams, &player.registration_token).unwrap_or_default();
        let full = |team: &Teams| team.players.len() == self.config.max_players_per_team as usize;
        let all_full = !teams.is_empty() && teams.values().all(full);
        let registered: usize = teams.values().map(|team| team.players.len()).sum();
        drop(teams);

        let mut game = self.lock_match()?;
        game.waiting.push((player.player_name.clone(), team));
        // A teammate may be registered without having joined yet; starting before it does
        // would leave it off the board.
        if !game.started && all_full && game.waiting.len() == registered {
            game.start(&self.config);
            info!("All teams are full, the game starts");
            self.game.1.notify_all();
        }
        Ok(())
    }

    /// Runs a subscribed player's game until it reaches the exit or leaves.
    fn play(&self, stream: &mut TcpStream, name: &str) -> GameResult<()> {
        let id = {
            let mut game = self.lock_match()?;
            while !game.started {
                game = self.game.1.wait(game).map_err(|e| GameError::ThreadError(e.to_string()))?;
            }
            let id = *game
                .players
                .get(name)
                .ok_or_else(|| GameError::ThreadError(format!("{name} is not on the board")))?;
            send_message(stream, &game.view(id))?;
            id
        };

        while let Ok(message) = receive_message(stream) {
            let Message::Action(action) = message else {
                warn!("{name} sent {message:?} during the game");
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
                return Ok(());
            }
        }
        info!("{name} left the game");
        Ok(())
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
                            send_message(&mut stream, &Message::SubscribePlayerResult(result))?;
                            if !joined {
                                continue;
                            }
                            // From here on this connection is the player's.
                            self.join_game(&player)?;
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
