use shared::func::{get_server_address, receive_message};
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
                        eprintln!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), std::io::Error> {
    let json = receive_message(&mut stream);
    match json {
        Ok(_) => (),
        Err(e) => eprintln!("Error receiving message: {}", e),
    }

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
