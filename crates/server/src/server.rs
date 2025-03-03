use rand::{Rng, rng};
use shared::errors::{GameError, GameResult};
use shared::logger::Logger;
use shared::messages::{
    Action, Challenge, Client, Hint, Message, MessageError, RadarView, RegisterTeamResult,
    RegistrationError, SubscribePlayerResult, Teams, receive_message, send_message,
};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub seed: u64,
    pub max_players_per_team: u8,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { host: "localhost".to_string(), port: 8080, seed: 0, max_players_per_team: 3 }
    }
}

pub struct GameServer {
    clients: Arc<Mutex<HashMap<String, Client>>>,
    teams: Arc<Mutex<HashMap<String, Teams>>>,
    config: ServerConfig,
    game_state: Arc<Mutex<GameState>>,
}

type ServerResult<T> = Result<T, RegistrationError>;

struct GameState {
    is_started: bool,
    teams: HashMap<String, Teams>,
    clients: HashMap<String, Client>,
    connections: HashMap<String, TcpStream>,
}

impl GameState {
    fn new() -> Self {
        Self {
            is_started: false,
            teams: HashMap::new(),
            clients: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    fn are_all_teams_full(&self, max_players_per_team: u8) -> bool {
        if self.teams.is_empty() {
            return false;
        }

        self.teams.values().all(|team| team.players.len() == max_players_per_team as usize)
    }

    fn notify_all_players(&mut self, message: &Message) -> GameResult<()> {
        for (_, stream) in self.connections.iter_mut() {
            send_message(stream, message)?
        }
        Ok(())
    }

    fn start_game(&mut self) -> GameResult<()> {
        if !self.is_started {
            self.is_started = true;

            self.notify_all_players(&Message::Hint(Hint::Secret(17)))?;

            let player_names: Vec<String> = self.connections.keys().cloned().collect();
            if !player_names.is_empty() {
                let random_index = rand::rng().random_range(0..player_names.len());
                let selected_player = player_names[random_index].clone();

                if let Some(stream) = self.connections.get_mut(&selected_player) {
                    send_message(stream, &Message::Challenge(Challenge::SecretSumModulo(23)))?;
                }
            }

            for (_, stream) in self.connections.iter_mut() {
                send_message(stream, &Message::Challenge(Challenge::SecretSumModulo(23)))?;
                send_message(
                    stream,
                    &Message::RadarView(RadarView("bieakcGa//+F8pa".to_string())),
                )?;
            }
        }
        Ok(())
    }
}

impl GameServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            teams: Arc::new(Mutex::new(HashMap::new())),
            config: config.clone(),
            game_state: Arc::new(Mutex::new(GameState::new())),
        }
    }

    pub fn run(&self, logger: &Logger) -> GameResult<()> {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&address).map_err(GameError::ConnectionError)?;

        logger.info(&format!("Server listening on {}", address));
        self.handle_connections(listener)
    }

    fn handle_connections(&self, listener: TcpListener) -> GameResult<()> {
        let mut thread_handles: Vec<std::thread::JoinHandle<GameResult<()>>> = Vec::new();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let server = self.clone();
                    let handle = std::thread::spawn(move || server.handle_message(stream));
                    thread_handles.push(handle);
                }
                Err(e) => return Err(GameError::ConnectionError(e)),
            }
        }

        for handle in thread_handles {
            match handle.join() {
                Ok(result) => result?,
                Err(e) => {
                    return Err(GameError::ThreadError(format!("Thread panicked: {:?}", e)));
                }
            }
        }

        Ok(())
    }

    fn handle_player_registration(
        &self,
        player: Client,
        stream: &mut TcpStream,
        logger: &Logger,
    ) -> GameResult<SubscribePlayerResult> {
        let result = self.register_client(player.clone(), logger);

        if matches!(result, SubscribePlayerResult::Ok) {
            let mut game_state =
                self.game_state.lock().map_err(|e| GameError::ThreadError(e.to_string()))?;

            let stream_clone =
                stream.try_clone().map_err(|e| GameError::ThreadError(e.to_string()))?;

            game_state.connections.insert(player.player_name.clone(), stream_clone);

            if !game_state.is_started
                && game_state.are_all_teams_full(self.config.max_players_per_team)
            {
                game_state.start_game()?;
            }
        }

        Ok(result)
    }

    fn register_client(&self, player: Client, logger: &Logger) -> SubscribePlayerResult {
        if player.player_name.is_empty() {
            return SubscribePlayerResult::Err(RegistrationError::InvalidName);
        }

        if player.registration_token.is_empty() {
            return SubscribePlayerResult::Err(RegistrationError::InvalidRegistrationToken);
        }

        let mut clients = match self.clients.lock() {
            Ok(clients) => clients,
            Err(_) => {
                return SubscribePlayerResult::Err(RegistrationError::ServerError);
            }
        };

        let mut teams = match self.teams.lock() {
            Ok(teams) => teams,
            Err(_) => {
                return SubscribePlayerResult::Err(RegistrationError::ServerError);
            }
        };

        if clients.contains_key(&player.player_name) {
            return SubscribePlayerResult::Err(RegistrationError::AlreadyRegistered);
        }

        let team_name = match self.find_team_by_token(&teams, &player.registration_token) {
            Some(name) => name,
            None => {
                return SubscribePlayerResult::Err(RegistrationError::InvalidRegistrationToken);
            }
        };

        if let Some(team) = teams.get_mut(&team_name) {
            if team.players.len() >= self.config.max_players_per_team as usize {
                return SubscribePlayerResult::Err(RegistrationError::TooManyPlayers);
            }

            team.players.push(player.clone());
            clients.insert(player.player_name.clone(), player.clone());

            logger.info(&format!(
                "{} registered successfully on team {}",
                player.player_name, team_name
            ));

            let game_state_result =
                self.game_state.lock().map_err(|_| RegistrationError::ServerError);

            match game_state_result {
                Ok(mut game_state) => {
                    game_state.clients.insert(player.player_name.clone(), player.clone());
                    if let Some(game_team) = game_state.teams.get_mut(&team_name) {
                        game_team.players.push(player.clone());
                    }
                }
                Err(e) => {
                    return SubscribePlayerResult::Err(e);
                }
            }

            SubscribePlayerResult::Ok
        } else {
            logger.error(&format!("Team {} not found", team_name));
            SubscribePlayerResult::Err(RegistrationError::ServerError)
        }
    }

    fn find_team_by_token(&self, teams: &HashMap<String, Teams>, token: &str) -> Option<String> {
        teams
            .iter()
            .find(|(_, team)| team.registration_token == token)
            .map(|(name, _)| name.clone())
    }

    fn register_team(&self, mut team: Teams, logger: &Logger) -> ServerResult<String> {
        let mut teams = self.teams.lock().map_err(|_| RegistrationError::ServerError)?;

        if teams.contains_key(&team.team_name) {
            return Err(RegistrationError::TeamAlreadyRegistered);
        }

        let token = self.generate_token();
        let team_name = team.team_name.clone();
        team.registration_token.clone_from(&token);

        teams.insert(team_name.clone(), team.clone());

        if let Ok(mut game_state) = self.game_state.lock() {
            game_state.teams.insert(team_name.clone(), team);
        }

        logger.info(&format!("Team {} registered successfully", team_name));
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

        format!("{}{}", timestamp, random_part)
    }

    fn handle_message(&self, mut stream: TcpStream) -> GameResult<()> {
        let logger = Logger::get_instance();
        let peer_address = stream.peer_addr().map_err(GameError::ConnectionError)?;

        loop {
            let message_result = receive_message(&mut stream);
            logger.debug(&format!("Received message from {}", peer_address));

            match message_result {
                Ok(message) => {
                    let response = match message {
                        Message::RegisterTeam(team) => {
                            let team = Teams {
                                team_name: team.name.clone(),
                                registration_token: String::new(),
                                max_players: self.config.max_players_per_team,
                                players: Vec::new(),
                                score: 0,
                            };

                            match self.register_team(team, logger) {
                                Ok(token) => Message::RegisterTeamResult(RegisterTeamResult::Ok {
                                    registration_token: token,
                                    expected_players: self.config.max_players_per_team,
                                }),
                                Err(err) => {
                                    logger.error(&format!(
                                        "Team registration failed for {}: {:?}",
                                        peer_address, err
                                    ));
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

                            match self.handle_player_registration(player, &mut stream, logger) {
                                Ok(result) => Message::SubscribePlayerResult(result),
                                Err(_) => Message::SubscribePlayerResult(
                                    SubscribePlayerResult::Err(RegistrationError::ServerError),
                                ),
                            }
                        }
                        Message::Action(Action::SolveChallenge { answer }) => {
                            logger.info(&format!(
                                "Received SolveChallenge from {}: Answer = {}",
                                peer_address, answer
                            ));

                            let is_correct = answer == "5";
                            if is_correct {
                                logger.info(&format!(
                                    "Challenge solved correctly by {}",
                                    peer_address
                                ));
                            } else {
                                logger.warn(&format!("Incorrect answer from {}", peer_address));
                            }
                            Message::RadarView(RadarView("bieakcGa//+F8pa".to_string()))
                        }
                        Message::Action(Action::MoveTo(_)) => {
                            logger.info(&format!("Received MoveTo from {}", peer_address));

                            break;
                        }
                        _ => {
                            logger.warn(&format!(
                                "Received invalid message type from {}",
                                peer_address
                            ));
                            Message::MessageError(MessageError {
                                message: "Invalid message type".to_string(),
                            })
                        }
                    };

                    if let Err(e) = send_message(&mut stream, &response) {
                        logger.error(&format!("Failed to send message to {}: {}", peer_address, e));
                        break;
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }

        logger.info(&format!("Connection closed for {}", peer_address));
        Ok(())
    }
}

impl Clone for GameServer {
    fn clone(&self) -> Self {
        Self {
            clients: Arc::clone(&self.clients),
            teams: Arc::clone(&self.teams),
            config: self.config.clone(),
            game_state: Arc::clone(&self.game_state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::str::FromStr;

    fn setup_test_environment() -> (GameServer, &'static Logger) {
        Logger::init(true);
        let logger = Logger::get_instance();
        let config = ServerConfig::default();
        let server = GameServer::new(config);
        (server, logger)
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
        let (server, logger) = setup_test_environment();
        let team = create_test_team("Test Team");
        let result = server.register_team(team, logger);
        assert!(result.is_ok());

        let teams = server.teams.lock().unwrap();
        assert!(teams.contains_key("Test Team"));
    }

    #[test]
    fn test_register_team_duplicate() {
        let (server, logger) = setup_test_environment();
        let team = create_test_team("Test Team");
        let team2 = create_test_team("Test Team");

        let _ = server.register_team(team, logger);
        let result = server.register_team(team2, logger);

        assert!(matches!(result, Err(RegistrationError::TeamAlreadyRegistered)));
    }

    #[test]
    fn test_generate_token() {
        let (server, _) = setup_test_environment();

        let token1 = server.generate_token();
        let token2 = server.generate_token();

        assert_ne!(token1, token2);
    }

    #[test]
    fn test_register_client_success() {
        let (server, logger) = setup_test_environment();
        let team = create_test_team("Test Team");
        let token = server.register_team(team, logger).unwrap();
        let client = create_test_client("Test Player", &token);

        let res = server.register_client(client, logger);
        assert!(matches!(res, SubscribePlayerResult::Ok));

        let clients = server.clients.lock().unwrap();
        assert!(clients.contains_key("Test Player"));
    }

    #[test]
    fn test_register_client_invalid_token() {
        let (server, logger) = setup_test_environment();
        let client = create_test_client("Test Player", "invalid_token");

        let result = server.register_client(client, logger);
        assert!(matches!(
            result,
            SubscribePlayerResult::Err(RegistrationError::InvalidRegistrationToken)
        ));
    }

    #[test]
    fn test_register_client_empty_name() {
        let (server, logger) = setup_test_environment();
        let team = create_test_team("Test Team");
        let token = server.register_team(team, logger).unwrap();
        let client = create_test_client("", &token);

        let result = server.register_client(client, logger);
        assert!(matches!(result, SubscribePlayerResult::Err(RegistrationError::InvalidName)));
    }

    #[test]
    fn test_register_multiple_clients_same_team() {
        let (server, logger) = setup_test_environment();
        let team = create_test_team("Test Team");
        let token = server.register_team(team, logger).unwrap();

        for i in 1..=3 {
            let client = create_test_client(&format!("Player{}", i), &token);
            let res = server.register_client(client, logger);
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

        let (server, logger) = setup_test_environment();
        let server = Arc::new(server);
        let team = create_test_team("Test Team");
        let token = server.register_team(team, logger).unwrap();

        let mut handles = vec![];
        for i in 0..3 {
            let server_clone = Arc::clone(&server);
            let token_clone = token.clone();

            handles.push(thread::spawn(move || {
                let client = create_test_client(&format!("Player{}", i), &token_clone);
                server_clone.register_client(client, logger)
            }));
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let success_count =
            results.iter().filter(|&r| matches!(r, SubscribePlayerResult::Ok)).count();

        assert_eq!(success_count, 3);
    }
}
