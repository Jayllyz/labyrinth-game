use clap::Parser;
use client::client::{ClientConfig, GameClient};
use shared::radar;
use shared::utils::print_error;

#[derive(Parser, Debug)]
#[command(name = "Labyrinth-client")]
#[command(version = "1.0")]
#[command(about = "Client for the Labyrinth game")]
struct Args {
    #[arg(
        long = "host-address",
        default_value = "localhost",
        help = "Server address to connect to."
    )]
    #[arg(help_heading = "SERVER OPTIONS")]
    host: String,

    #[arg(short, long, default_value = "8778", help = "Port to connect to.")]
    #[arg(help_heading = "SERVER OPTIONS")]
    #[arg(value_parser = clap::value_parser!(u16).range(1024..=u16::MAX as i64))]
    port: u16,

    #[arg(short, long, default_value = "5", help = "Number of retries to connect to the server.")]
    #[arg(help_heading = "SERVER OPTIONS")]
    retries: u8,

    #[arg(short, long, default_value = "Groupe1", help = "Team name.")]
    #[arg(help_heading = "PLAYER OPTIONS")]
    team: String,

    #[arg(long, help = "Number of players in the team.")]
    #[arg(help_heading = "PLAYER OPTIONS")]
    players: Option<u8>,

    #[arg(long, help = "Token to register the player.")]
    #[arg(help_heading = "PLAYER OPTIONS")]
    token: Option<String>,

    #[arg(long, help = "Run the client in offline mode.")]
    offline: bool,
}

fn main() {
    let args = Args::parse();

    let config = ClientConfig {
        server_addr: format!("{}:{}", args.host, args.port),
        team_name: args.team,
        token: args.token,
    };

    if args.offline {
        println!("Running in offline mode (no connection to the server)");
        println!("Not implemented yet, exiting...");
        let decoded = radar::decode_base64("jivbQjIad/apapa");
        let data = radar::extract_data(&decoded);
        println!("{:?}", data.0);
        println!("{:?}", data.1);
        println!("{:?}", data.2);
        return;
    }

    let client = GameClient::new(config);
    let agents_count = args.players.unwrap_or(3);
    match client.run(args.retries, agents_count) {
        Ok(_) => println!("All agents have found the exit!"),
        Err(e) => {
            print_error(&format!("{}", e));
        }
    }
}
