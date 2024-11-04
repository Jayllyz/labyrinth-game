use shared::messages::{
    receive_message, send_message, Client, Message, SubscribeError, SubscribeResult, Teams, Welcome,
};
use shared::utils::{print_log, Color};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub seed: u64,
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
                Err(e) => eprintln!("Failed to establish connection: {}", e),
            }
        }
    }

    fn register_client(&self, player: Client) -> SubscribeResult {
        if player.player_name.is_empty() {
            return SubscribeResult::Err(SubscribeError::InvalidName);
        }

        let mut clients = self.clients.lock().expect("Failed to lock clients");

        if clients.contains_key(player.player_name.as_str()) {
            return SubscribeResult::Err(SubscribeError::AlreadyRegistered);
        }

        let client = Client {
            player_name: player.player_name.clone(),
            team_name: player.team_name.clone(),
            address: player.address,
            moves_count: 0,
            score: 0,
        };

        clients.insert(player.player_name.clone(), client);

        let mut teams = self.teams.lock().expect("Failed to lock teams");
        let team = teams.entry(player.team_name.clone()).or_insert_with(|| Teams {
            team_name: player.team_name.clone(),
            players: Vec::new(),
            score: 0,
        });

        team.players.push(player.clone());

        print_log(
            &format!("{} registered successfully on team {}", player.player_name, player.team_name),
            Color::Green,
        );
        SubscribeResult::Ok
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        while let Ok(message) = receive_message(&mut stream) {
            let response = match message {
                Message::Hello => Message::Welcome(Welcome { version: 1 }),

                Message::Subscribe(subscribe) => {
                    let player = Client {
                        player_name: subscribe.name,
                        team_name: subscribe.team,
                        address: stream.peer_addr().expect("Failed to get peer address"),
                        moves_count: 0,
                        score: 0,
                    };
                    Message::SubscribeResult(self.register_client(player))
                }

                _ => Message::MessageError(shared::messages::MessageError {
                    message: "Invalid message".to_string(),
                }),
            };
            send_message(&mut stream, response)
        }
    }
}

impl Default for GameServer {
    fn default() -> Self {
        Self::new(ServerConfig { host: "127.0.0.1".to_string(), port: 8080, seed: 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client(name: &str, team: &str) -> Client {
        Client {
            player_name: name.to_string(),
            team_name: team.to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            moves_count: 0,
            score: 0,
        }
    }

    #[test]
    fn test_register_client() {
        let config = ServerConfig { host: "127.0.0.1".to_string(), port: 8080, seed: 0 };
        let server = GameServer::new(config);
        let player = create_test_client("Player1", "Team1");

        assert!(matches!(server.register_client(player.clone()), SubscribeResult::Ok));

        let clients = server.clients.lock().unwrap();
        assert!(clients.contains_key(&player.player_name));

        let teams = server.teams.lock().unwrap();
        assert!(teams.contains_key(&player.team_name));
        assert_eq!(teams[&player.team_name].players.len(), 1);
    }

    #[test]
    fn test_register_duplicate_client() {
        let config = ServerConfig { host: "127.0.0.1".to_string(), port: 8080, seed: 0 };
        let server = GameServer::new(config);
        let player = create_test_client("Player1", "Team1");

        assert!(matches!(server.register_client(player.clone()), SubscribeResult::Ok));
        assert!(matches!(
            server.register_client(player.clone()),
            SubscribeResult::Err(SubscribeError::AlreadyRegistered)
        ));
    }

    #[test]
    fn test_register_invalid_name() {
        let config = ServerConfig { host: "127.0.0.1".to_string(), port: 8080, seed: 0 };
        let server = GameServer::new(config);
        let player = create_test_client("", "Team1");

        assert!(matches!(
            server.register_client(player),
            SubscribeResult::Err(SubscribeError::InvalidName)
        ));
    }

    #[test]
    fn test_multiple_players_same_team() {
        let config = ServerConfig { host: "127.0.0.1".to_string(), port: 8080, seed: 0 };
        let server = GameServer::new(config);
        let player1 = create_test_client("Player1", "Team1");
        let player2 = create_test_client("Player2", "Team1");

        assert!(matches!(server.register_client(player1.clone()), SubscribeResult::Ok));
        assert!(matches!(server.register_client(player2.clone()), SubscribeResult::Ok));

        let teams = server.teams.lock().unwrap();
        let team = teams.get("Team1").unwrap();
        assert_eq!(team.players.len(), 2);
        assert!(team.players.iter().any(|p| p.player_name == "Player1"));
        assert!(team.players.iter().any(|p| p.player_name == "Player2"));
    }

    #[test]
    fn test_players_different_teams() {
        let config = ServerConfig { host: "127.0.0.1".to_string(), port: 8080, seed: 0 };
        let server = GameServer::new(config);
        let player1 = create_test_client("Player1", "Team1");
        let player2 = create_test_client("Player2", "Team2");

        assert!(matches!(server.register_client(player1.clone()), SubscribeResult::Ok));
        assert!(matches!(server.register_client(player2.clone()), SubscribeResult::Ok));

        let teams = server.teams.lock().unwrap();
        assert_eq!(teams.len(), 2);
        assert_eq!(teams["Team1"].players.len(), 1);
        assert_eq!(teams["Team2"].players.len(), 1);
    }
}
