use shared::{
    messages::{receive_message, send_message, Message},
    utils::{get_server_address, print_error, print_log, Color},
};
use std::net::{TcpListener, TcpStream};

fn main() {
    let server_address = get_server_address();
    let listener = TcpListener::bind(server_address.clone()).expect("Failed to bind to address");
    print_log(&format!("Server started on {}", server_address), Color::Blue);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => print_error(&format!("Failed to establish connection: {}", e)),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    while let Ok(message) = receive_message(&mut stream) {
        let response = match message {
            Message::Hello => Message::Welcome(shared::messages::Welcome { version: 1 }),
            Message::Subscribe(subscribe) => {
                if subscribe.name.is_empty() {
                    Message::SubscribeResult(shared::messages::SubscribeResult::Err(
                        shared::messages::SubscribeError::InvalidName,
                    ))
                } else {
                    Message::SubscribeResult(shared::messages::SubscribeResult::Ok)
                }
                //todo already registered user
            }
            _ => Message::MessageError(shared::messages::MessageError {
                message: "Invalid message".to_string(),
            }),
        };
        send_message(&mut stream, response)
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
