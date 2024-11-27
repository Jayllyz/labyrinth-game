use rand::Rng;
use shared::messages::{
    receive_message, send_message, Client, Message, RegisterTeamResult, RegistrationError,
    SubscribePlayerResult, Teams,
};
use shared::utils::{print_error, print_log, Color};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub seed: u64,
    pub max_players_per_team: u8,
}

pub struct GameServer {
    clients: Arc<Mutex<HashMap<String, Client>>>,
    teams: Arc<Mutex<HashMap<String, Teams>>>,
    config: ServerConfig,
}

impl GameServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            teams: Arc::new(Mutex::new(HashMap::new())),
            config: config.clone(),
        }
    }

    pub fn run(&self) {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let listener =
            std::net::TcpListener::bind(address.clone()).expect("Failed to bind to address");
        print_log(&format!("Server started on {}", address), Color::Reset);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let clients = Arc::clone(&self.clients);
                    let teams = Arc::clone(&self.teams);
                    let config = self.config.clone();
                    std::thread::spawn(move || {
                        Self::handle_connection(&GameServer { clients, teams, config }, stream)
                    });
                }
                Err(e) => print_error(&format!("Failed to accept connection: {}", e)),
            }
        }
    }

    fn register_client(&self, player: Client) -> SubscribePlayerResult {
        if player.player_name.is_empty() {
            return SubscribePlayerResult::Err(RegistrationError::InvalidName);
        }

        if player.registration_token.is_empty() {
            return SubscribePlayerResult::Err(RegistrationError::InvalidRegistrationToken);
        }

        let mut clients = match self.clients.lock() {
            Ok(clients) => clients,
            Err(_) => return SubscribePlayerResult::Err(RegistrationError::ServerError),
        };
        let mut teams = match self.teams.lock() {
            Ok(teams) => teams,
            Err(_) => return SubscribePlayerResult::Err(RegistrationError::ServerError),
        };

        if clients.contains_key(player.player_name.as_str()) {
            return SubscribePlayerResult::Err(RegistrationError::AlreadyRegistered);
        }

        let team_name = match teams
            .iter()
            .find(|(_, team)| team.registration_token == player.registration_token)
        {
            Some((name, _)) => name.clone(),
            None => {
                return SubscribePlayerResult::Err(RegistrationError::InvalidRegistrationToken);
            }
        };

        if let Some(team) = teams.get_mut(&team_name) {
            if team.players.len() >= self.config.max_players_per_team as usize {
                return SubscribePlayerResult::Err(RegistrationError::TooManyPlayers);
            }

            team.players.push(player.clone());

            let client = Client {
                player_name: player.player_name.clone(),
                team_name: player.team_name.clone(),
                address: player.address,
                moves_count: 0,
                score: 0,
                registration_token: player.registration_token.clone(),
            };

            clients.insert(player.player_name.clone(), client);

            print_log(
                &format!(
                    "{} registered successfully on team {}",
                    player.player_name, player.team_name
                ),
                Color::Green,
            );

            SubscribePlayerResult::Ok
        } else {
            SubscribePlayerResult::Err(RegistrationError::ServerError)
        }
    }

    fn register_team(&self, team: Teams) -> Result<String, RegistrationError> {
        let mut teams = match self.teams.lock() {
            Ok(teams) => teams,
            Err(_) => return Err(RegistrationError::ServerError),
        };

        if teams.contains_key(&team.team_name) {
            return Err(RegistrationError::TeamAlreadyRegistered);
        }

        let token: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let team = Teams {
            team_name: team.team_name,
            players: Vec::new(),
            score: 0,
            registration_token: token.clone(),
        };

        teams.insert(team.team_name.clone(), team);

        Ok(token)
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        while let Ok(message) = receive_message(&mut stream) {
            let response = match message {
                Message::RegisterTeam(team) => {
                    let team = Teams {
                        team_name: team.name,
                        players: Vec::new(),
                        score: 0,
                        registration_token: String::new(),
                    };

                    match self.register_team(team) {
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
                        address: stream.peer_addr().expect("Failed to get peer address"),
                        moves_count: 0,
                        score: 0,
                        registration_token: subscribe.registration_token,
                    };
                    Message::SubscribePlayerResult(self.register_client(player))
                }

                _ => Message::MessageError(shared::messages::MessageError {
                    message: "Invalid message".to_string(),
                }),
            };

            match send_message(&mut stream, &response) {
                Ok(_) => {}
                Err(e) => {
                    print_log(&format!("[warning] - Failed to send message: {}", e), Color::Orange);
                }
            }

            if matches!(response, Message::RegisterTeamResult(RegisterTeamResult::Ok { .. })) {
                // If the team registration was successful, we can break the connection.
                break;
            }
        }
    }
}

impl Default for GameServer {
    fn default() -> Self {
        Self::new(ServerConfig {
            host: "localhost".to_string(),
            port: 8080,
            seed: 0,
            max_players_per_team: 3,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::str::FromStr;

    fn setup_server() -> GameServer {
        GameServer::new(ServerConfig {
            host: "localhost".to_string(),
            port: 8080,
            seed: 0,
            max_players_per_team: 3,
        })
    }

    #[test]
    fn test_register_team_success() {
        let server = setup_server();
        let team = Teams {
            team_name: "Test Team".to_string(),
            players: Vec::new(),
            score: 0,
            registration_token: String::new(),
        };

        let result = server.register_team(team);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 10);

        let teams = server.teams.lock().unwrap();
        assert!(teams.contains_key("Test Team"));
    }

    #[test]
    fn test_register_team_duplicate() {
        let server = setup_server();
        let team = Teams {
            team_name: "Test Team".to_string(),
            players: Vec::new(),
            score: 0,
            registration_token: String::new(),
        };

        let _ = server.register_team(team);

        let team = Teams {
            team_name: "Test Team".to_string(),
            players: Vec::new(),
            score: 0,
            registration_token: String::new(),
        };

        let result = server.register_team(team);

        assert!(matches!(result, Err(RegistrationError::TeamAlreadyRegistered)));
    }

    #[test]
    fn test_register_client_success() {
        let server = setup_server();
        let team = Teams {
            team_name: "Test Team".to_string(),
            players: Vec::new(),
            score: 0,
            registration_token: String::new(),
        };

        let token = server.register_team(team).unwrap();
        println!("{:?}", token);

        let client = Client {
            player_name: "Test Player".to_string(),
            team_name: "Test Team".to_string(),
            address: SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            moves_count: 0,
            score: 0,
            registration_token: token,
        };

        let result = server.register_client(client);
        assert!(matches!(result, SubscribePlayerResult::Ok));

        let clients = server.clients.lock().unwrap();
        assert!(clients.contains_key("Test Player"));
    }

    #[test]
    fn test_register_client_invalid_token() {
        let server = setup_server();
        let client = Client {
            player_name: "Test Player".to_string(),
            team_name: "Test Team".to_string(),
            address: SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            moves_count: 0,
            score: 0,
            registration_token: "invalid_token".to_string(),
        };

        let result = server.register_client(client);
        assert!(matches!(
            result,
            SubscribePlayerResult::Err(RegistrationError::InvalidRegistrationToken)
        ));
    }

    #[test]
    fn test_register_client_team_full() {
        let server = setup_server();

        let team = Teams {
            team_name: "Test Team".to_string(),
            players: Vec::new(),
            score: 0,
            registration_token: String::new(),
        };

        let token = server.register_team(team).unwrap();

        let base_client = Client {
            team_name: "Test Team".to_string(),
            address: SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            moves_count: 0,
            score: 0,
            registration_token: token.clone(),
            player_name: String::new(),
        };

        for i in 1..=3 {
            let mut client = base_client.clone();
            client.player_name = format!("Player{}", i);
            let result = server.register_client(client);
            assert!(matches!(result, SubscribePlayerResult::Ok));
        }

        let mut client = base_client.clone();
        client.player_name = "Player4".to_string();
        let result = server.register_client(client);

        assert!(matches!(result, SubscribePlayerResult::Err(RegistrationError::TooManyPlayers)));
    }

    #[test]
    fn test_register_client_duplicate_name() {
        let server = setup_server();

        let team = Teams {
            team_name: "Test Team".to_string(),
            players: Vec::new(),
            score: 0,
            registration_token: String::new(),
        };

        let token = server.register_team(team).unwrap();

        let client = Client {
            player_name: "Test Player".to_string(),
            team_name: "Test Team".to_string(),
            address: SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            moves_count: 0,
            score: 0,
            registration_token: token.clone(),
        };

        let _ = server.register_client(client.clone());
        let result = server.register_client(client);

        assert!(matches!(result, SubscribePlayerResult::Err(RegistrationError::AlreadyRegistered)));
    }
}
