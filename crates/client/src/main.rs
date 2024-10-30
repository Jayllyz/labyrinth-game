use shared::func::{get_server_address, receive_message, send_message};
use std::net::TcpStream;

fn main() {
    let server_address = get_server_address();
    let mut max_retries = 5;
    let mut stream = loop {
        match TcpStream::connect(server_address.clone()) {
            Ok(stream) => break stream,
            Err(e) => {
                eprintln!("Failed to connect to server: {}", e);
                max_retries -= 1;
                std::thread::sleep(std::time::Duration::from_secs(1));
                if max_retries == 0 {
                    eprintln!("Max retries reached, exiting...");
                    std::process::exit(1);
                }
                continue;
            }
        }
    };

    let msg = shared::messages::Message::Hello;
    send_message(&mut stream, msg);
    let _ = receive_message(&mut stream).expect("Failed to receive message");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello() {
        assert_eq!("Hello, client!", "Hello, client!");
    }
}
