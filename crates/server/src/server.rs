use rand::Rng;
use shared::errors::{GameError, GameResult};
use shared::logger::Logger;
use shared::messages::{
    receive_message, send_message, Client, Message, RegisterTeamResult, RegistrationError,
    SubscribePlayerResult, Teams,
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
}

type ServerResult<T> = Result<T, RegistrationError>;

impl GameServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            teams: Arc::new(Mutex::new(HashMap::new())),
            config: config.clone(),
        }
    }

    pub fn run(&self, logger: &Logger) -> GameResult<()> {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&address).map_err(|e| {
            logger.error(&format!("Failed to bind to address {}: {}", address, e));
            GameError::ConnectionError(e)
        })?;

        logger.info(&format!("Server listening on {}", address));
        self.handle_connections(listener)
    }

    fn handle_connections(&self, listener: TcpListener) -> GameResult<()> {
        let mut thread_handles: Vec<std::thread::JoinHandle<GameResult<()>>> = Vec::new();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let clients = Arc::clone(&self.clients);
                    let teams = Arc::clone(&self.teams);
                    let config = self.config.clone();

                    let handle = std::thread::spawn(move || {
                        let server = GameServer { clients, teams, config };
                        server.handle_connection(stream)?;
                        Ok(())
                    });

                    thread_handles.push(handle);
                }
                Err(e) => {
                    return Err(GameError::ConnectionError(e));
                }
            }
        }

        for handle in thread_handles {
            match handle.join() {
                Ok(result) => result?,
                Err(e) => {
                    let err = GameError::ThreadError(format!("Thread panicked: {:?}", e));
                    return Err(err);
                }
            }
        }

        Ok(())
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
            Err(e) => {
                logger.error(&format!("Failed to lock clients: {}", e));
                return SubscribePlayerResult::Err(RegistrationError::ServerError);
            }
        };

        let mut teams = match self.teams.lock() {
            Ok(teams) => teams,
            Err(e) => {
                logger.error(&format!("Failed to lock teams: {}", e));
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

        self.add_player_to_team(&mut teams, &team_name, player, &mut clients, logger)
    }

    fn find_team_by_token(&self, teams: &HashMap<String, Teams>, token: &str) -> Option<String> {
        teams
            .iter()
            .find(|(_, team)| team.registration_token == token)
            .map(|(name, _)| name.clone())
    }

    fn add_player_to_team(
        &self,
        teams: &mut HashMap<String, Teams>,
        team_name: &str,
        player: Client,
        clients: &mut HashMap<String, Client>,
        logger: &Logger,
    ) -> SubscribePlayerResult {
        if let Some(team) = teams.get_mut(team_name) {
            if team.players.len() >= self.config.max_players_per_team as usize {
                logger.warn(&format!("Team {} is full", team_name));
                return SubscribePlayerResult::Err(RegistrationError::TooManyPlayers);
            }

            team.players.push(player.clone());
            clients.insert(player.player_name.clone(), player.clone());

            logger.info(&format!(
                "{} registered successfully on team {}",
                player.player_name, team_name
            ));

            SubscribePlayerResult::Ok
        } else {
            logger.error(&format!("Team {} not found", team_name));
            SubscribePlayerResult::Err(RegistrationError::ServerError)
        }
    }

    fn register_team(&self, team: Teams, logger: &Logger) -> ServerResult<String> {
        let mut teams = self.teams.lock().map_err(|e| {
            logger.error(&format!("Failed to lock teams: {}", e));
            RegistrationError::ServerError
        })?;

        if teams.contains_key(&team.team_name) {
            logger.warn(&format!("Team {} already registered", team.team_name));
            return Err(RegistrationError::TeamAlreadyRegistered);
        }

        let token = self.generate_token();
        let team_name = team.team_name.clone();
        let team = Teams {
            team_name: team_name.clone(),
            players: Vec::new(),
            score: 0,
            registration_token: token.clone(),
        };

        teams.insert(team_name.clone(), team);
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
            rand::rng().sample_iter(&rand::distr::Alphanumeric).take(16).map(char::from).collect();

        format!("{}{}", timestamp, random_part)
    }

    fn handle_connection(&self, mut stream: TcpStream) -> GameResult<()> {
        let logger = Logger::get_instance();
        while let Ok(message) = receive_message(&mut stream) {
            let response = match message {
                Message::RegisterTeam(team) => {
                    let team = Teams {
                        team_name: team.name,
                        players: Vec::new(),
                        score: 0,
                        registration_token: String::new(),
                    };

                    match self.register_team(team, logger) {
                        Ok(token) => Message::RegisterTeamResult(RegisterTeamResult::Ok {
                            expected_players: self.config.max_players_per_team,
                            registration_token: token,
                        }),
                        Err(e) => Message::RegisterTeamResult(RegisterTeamResult::Err(e)),
                    }
                }
                Message::SubscribePlayer(subscribe) => {
                    let player = Client {
                        player_name: subscribe.name,
                        team_name: String::new(),
                        address: stream.peer_addr()?,
                        registration_token: subscribe.registration_token,
                    };
                    Message::SubscribePlayerResult(self.register_client(player, logger))
                }
                _ => {
                    logger.warn("Received invalid message type");
                    Message::MessageError(shared::messages::MessageError {
                        message: "Invalid message".to_string(),
                    })
                }
            };

            send_message(&mut stream, &response)?;

            if matches!(response, Message::RegisterTeamResult(RegisterTeamResult::Ok { .. })) {
                break;
            }
        }
        Ok(())
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
    fn test_register_team_generates_unique_tokens() {
        let (server, logger) = setup_test_environment();
        let team1 = create_test_team("Team1");
        let team2 = create_test_team("Team2");

        let token1 = server.register_team(team1, logger).unwrap();
        let token2 = server.register_team(team2, logger).unwrap();

        assert_ne!(token1, token2);
    }

    #[test]
    fn test_register_client_success() {
        let (server, logger) = setup_test_environment();
        let team = create_test_team("Test Team");
        let token = server.register_team(team, logger).unwrap();
        let client = create_test_client("Test Player", &token);

        let result = server.register_client(client, logger);
        assert!(matches!(result, SubscribePlayerResult::Ok));

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
            let result = server.register_client(client, logger);
            assert!(matches!(result, SubscribePlayerResult::Ok));
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
