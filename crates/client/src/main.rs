use shared::{
    messages::*,
    utils::{get_player_name, get_server_address, get_team_name},
};
use std::net::TcpStream;

fn connect_to_server(mut max_retries: u32) -> TcpStream {
    let server_address = get_server_address();
    loop {
        match TcpStream::connect(&server_address) {
            Ok(stream) => return stream,
            Err(e) => {
                eprintln!("Failed to connect to server: {}", e);
                max_retries -= 1;
                std::thread::sleep(std::time::Duration::from_secs(2));
                if max_retries == 0 {
                    eprintln!("Max retries reached, exiting...");
                    std::process::exit(1);
                }
                continue;
            }
        }
    }
}

fn handle_server_message(stream: &mut TcpStream, message: Message) {
    match message {
        Message::Welcome(..) => {
            let subscribe = Subscribe { name: get_player_name(), team: get_team_name() };
            send_message(stream, Message::Subscribe(subscribe));
        }
        Message::SubscribeResult(result) => match result {
            SubscribeResult::Ok => {
                eprintln!("Subscribed successfully");
                std::process::exit(1);
            }
            SubscribeResult::Err(err) => {
                eprintln!("Subscribe error: {:?}", err);
            }
        },
        Message::MessageError(err) => {
            eprintln!("Error: {}", err.message);
        }
        _ => {
            eprintln!("Unexpected message, exiting...");
            std::process::exit(1);
        }
    }
}

fn main() {
    println!("Client started, connecting to server...");
    let mut stream = connect_to_server(5);

    send_message(&mut stream, Message::Hello);
    while let Ok(message) = receive_message(&mut stream) {
        handle_server_message(&mut stream, message);
    }
}
