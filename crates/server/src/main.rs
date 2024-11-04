use clap::Parser;
use server::server::GameServer;

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
        default_value = "127.0.0.1",
        help = "Address allowed to connect to."
    )]
    host: String,
}

fn main() {
    let args = Args::parse();
    let address = format!("{}:{}", args.host, args.port);

    let server = GameServer::new();
    server.run(&address);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let args = Args::try_parse_from(["test"]).unwrap();
        assert_eq!(args.port, 8778);
        assert_eq!(args.host, "127.0.0.1");
    }

    #[test]
    fn test_custom_port() {
        let args = Args::try_parse_from(["test", "--port", "8080"]).unwrap();
        assert_eq!(args.port, 8080);
        assert_eq!(args.host, "127.0.0.1");
    }
}
