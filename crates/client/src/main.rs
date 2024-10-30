use shared::func::{get_server_address, send_message};
use std::net::TcpStream;

fn main() {
    let server_address = get_server_address();
    let mut stream = TcpStream::connect(server_address).expect("Failed to connect to server");

    send_message(&mut stream, shared::messages::Message::Hello);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello() {
        assert_eq!("Hello, client!", "Hello, client!");
    }
}
