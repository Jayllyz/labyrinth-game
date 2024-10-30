use shared::{
    func::{get_server_address, receive_message, send_message},
    messages::{Message, MessageError},
};
use std::net::{TcpListener, TcpStream};

fn main() {
    let server_address = get_server_address();
    let listener = TcpListener::bind(server_address.clone()).expect("Failed to bind to address");
    println!("Server listening on: {:?}", server_address);

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
    let msg = receive_message(&mut stream)?;

    let response = match msg {
        Message::Hello => Message::Welcome(shared::messages::Welcome { version: 1 }),
        _ => Message::MessageError(MessageError { message: "Invalid message".to_string() }),
    };

    send_message(&mut stream, response);

    Ok(())
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
