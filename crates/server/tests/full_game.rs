//! Our client plays whole games against our server, in one process.

use client::{
    client::{ClientConfig, GameClient},
    instructions::Algorithm,
};
use server::server::{GameServer, ServerConfig};
use std::{net::TcpListener, sync::mpsc, thread, time::Duration};

fn start_server(config: ServerConfig) -> (GameServer, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind a free port");
    let address = listener.local_addr().expect("local address").to_string();
    let server = GameServer::new(config);
    let running = server.clone();
    thread::spawn(move || running.serve(listener));
    (server, address)
}

fn play(address: &str, team: &str, players: u8, algorithm: Algorithm) -> mpsc::Receiver<bool> {
    let (done, finished) = mpsc::channel();
    let config = ClientConfig {
        server_addr: address.to_string(),
        team_name: team.to_string(),
        move_delay: Duration::ZERO,
        seed: 1,
    };
    thread::spawn(move || {
        let result = GameClient::new(config).run(1, players, None, algorithm);
        let _ = done.send(result.is_ok());
    });
    finished
}

#[test]
fn test_two_teams_escape_from_our_server() {
    let config = ServerConfig {
        seed: 7,
        max_players_per_team: 2,
        width: 8,
        height: 8,
        ..ServerConfig::default()
    };
    let (server, address) = start_server(config);

    let first = play(&address, "Red", 2, Algorithm::Tremeaux);
    let second = play(&address, "Blue", 2, Algorithm::WallFollower);

    for team in [first, second] {
        assert_eq!(team.recv_timeout(Duration::from_secs(20)), Ok(true));
    }
    // The client also stops when the server drops it, so check the server saw the exits.
    assert_eq!(server.players_out(), 4);
}
