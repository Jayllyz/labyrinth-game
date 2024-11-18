use clap::Parser;
use server::server::{GameServer, ServerConfig};
use shared::utils::{print_log, Color};

#[derive(Parser, Debug)]
#[command(name = "Labyrinth-server")]
#[command(version = "1.0")]
#[command(about = "Server for the Labyrinth game", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "8778", help = "Port to listen to.")]
    #[arg(value_parser = clap::value_parser!(u16).range(1024..=65535))]
    port: u16,

    #[arg(
        long = "host-address",
        default_value = "localhost",
        help = "Address allowed to connect to."
    )]
    host: String,

    #[arg(short, long, help = "Seed for the maze generation.")]
    seed: Option<u64>,
}

fn main() {
    let args = Args::parse();
    let seed = args.seed.unwrap_or_else(rand::random);
    let config = ServerConfig { host: args.host, port: args.port, seed };
    print_log(&format!("seed {}", seed), Color::Blue);

    let server = GameServer::new(config);
    server.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let args = Args::try_parse_from(["test"]).unwrap();
        assert_eq!(args.port, 8778);
        assert_eq!(args.host, "localhost");
    }

    #[test]
    fn test_custom_port() {
        let args = Args::try_parse_from(["test", "--port", "8080"]).unwrap();
        assert_eq!(args.port, 8080);
        assert_eq!(args.host, "localhost");
    }
}
