use shared::{
    func::{get_player_name, get_server_address, receive_message, send_message},
    messages::*,
};
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

    send_message(&mut stream, Message::Hello);
    loop {
        let response = receive_message(&mut stream).expect("Failed to receive message");

        match response {
            Message::Welcome(..) => {
                let subscribe = Subscribe { name: get_player_name() };
                send_message(&mut stream, Message::Subscribe(subscribe));
            }
            Message::SubscribeResult(result) => match result {
                SubscribeResult::Ok => {
                    eprintln!("Subscribed successfully");
                    break;
                }
                SubscribeResult::Err(err) => {
                    eprintln!("Subscribe error: {:?}", err);
                    break;
                }
            },
            Message::MessageError(err) => {
                eprintln!("Error: {}", err.message);
                break;
            }
            _ => {
                eprintln!("Unexpected message: {:?}", response);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello() {
        assert_eq!("Hello, client!", "Hello, client!");
    }
}
