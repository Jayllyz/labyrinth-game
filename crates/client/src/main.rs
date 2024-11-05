use clap::Parser;
use client::client::{ClientConfig, GameClient};

#[derive(Parser, Debug)]
#[command(name = "Labyrinth-client")]
#[command(version = "1.0")]
#[command(about = "Client for the Labyrinth game")]
struct Args {
    #[arg(
        long = "host-address",
        default_value = "127.0.0.1",
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

    #[arg(short, long, default_value = "Player1", help = "Player name.")]
    #[arg(help_heading = "PLAYER OPTIONS")]
    name: String,

    #[arg(short, long, default_value = "Team1", help = "Team name.")]
    #[arg(help_heading = "PLAYER OPTIONS")]
    team: String,

    #[arg(long, help = "Run the client in offline mode.")]
    offline: bool,
}

fn main() {
    let args = Args::parse();

    let config = ClientConfig {
        server_addr: format!("{}:{}", args.host, args.port),
        player_name: args.name,
        team_name: args.team,
    };

    if args.offline {
        println!("Running in offline mode (no connection to the server)");
        println!("Not implemented yet, exiting...");
        return;
    }

    let client = GameClient::new(config);
    client.run(args.retries);
}
