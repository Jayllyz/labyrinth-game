use shared::{
    func::{get_server_address, receive_message, send_message},
    messages::Message,
};
use std::net::{TcpListener, TcpStream};

fn main() {
    let server_address = get_server_address();
    let listener = TcpListener::bind(server_address.clone()).expect("Failed to bind to address");
    println!("Server listening on: {:?}", server_address);
    listener.set_ttl(100).expect("could not set TTL");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| {
                    let res = handle_connection(stream);
                    if let Err(e) = res {
                        eprintln!("{}", e);
                    }
                });
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let response = receive_message(&mut stream);

    match response {
        Ok(Message::Hello) => {
            send_message(
                &mut stream,
                Message::Welcome(shared::messages::Welcome { version: 1 }),
            );
            Ok(())
        },
        Ok(_) => {
            send_message(
                &mut stream,
                Message::MessageError(shared::messages::MessageError {
                    message: "Invalid message".to_string(),
                }),
            );
            Ok(())
        },
        Err(e) => {
            Err(e.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_server() {
        let server_address = get_server_address();
        let listener =
            TcpListener::bind(server_address.clone()).expect("Failed to bind to address");
        assert!(listener.local_addr().is_ok());
    }
}
