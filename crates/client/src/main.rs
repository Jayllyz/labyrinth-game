use clap::Parser;
use client::client::{ClientConfig, GameClient};

#[derive(Parser, Debug)]
#[command(name = "Labyrinth-client")]
#[command(version = "1.0")]
#[command(about = "Client for the Labyrinth game")]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    #[arg(help_heading = "SERVER OPTIONS")]
    host: String,

    #[arg(short, long, default_value = "7878")]
    #[arg(help_heading = "SERVER OPTIONS")]
    #[arg(value_parser = clap::value_parser!(u16).range(1024..=65535))]
    port: u16,

    #[arg(short, long, default_value = "5")]
    #[arg(help_heading = "SERVER OPTIONS")]
    retries: u32,

    #[arg(short, long, default_value = "Player1")]
    #[arg(help_heading = "PLAYER OPTIONS")]
    name: String,

    #[arg(short, long, default_value = "Team1")]
    #[arg(help_heading = "PLAYER OPTIONS")]
    team: String,
}

fn main() {
    let args = Args::parse();

    let config = ClientConfig {
        server_addr: format!("{}:{}", args.host, args.port),
        player_name: args.name,
        team_name: args.team,
    };

    let client = GameClient::new(config);
    const MAX_RETRIES: u32 = 5;
    client.run(MAX_RETRIES);
}
