use server::server::GameServer;
use shared::utils::get_server_address;

fn main() {
    let server = GameServer::new();
    let server_address = get_server_address();
    server.run(&server_address);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn bind_server() {
        let server_address = get_server_address();
        let listener =
            TcpListener::bind(server_address.clone()).expect("Failed to bind to address");
        assert!(listener.local_addr().is_ok());
    }
}
