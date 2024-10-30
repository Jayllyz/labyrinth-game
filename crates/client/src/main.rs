use shared::func::{get_server_address, send_message};
use std::{io::Write, net::TcpStream};

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

    send_message(&mut stream, shared::messages::Message::Hello);
    stream.flush().expect("Failed to flush stream");
    drop(stream);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello() {
        assert_eq!("Hello, client!", "Hello, client!");
    }
}
