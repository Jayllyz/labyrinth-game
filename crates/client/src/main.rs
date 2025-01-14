use clap::Parser;
use client::client::{ClientConfig, GameClient};
use client::tui;
use shared::logger::Logger;

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

    #[arg(long, help = "Number of players in the team.", default_value = "3")]
    #[arg(help_heading = "PLAYER OPTIONS")]
    players: u8,

    #[arg(long, help = "Run the client in offline mode.")]
    offline: bool,

    #[arg(long, help = "Enable debug logs.", default_value = "false")]
    debug: bool,

    #[arg(long, help = "Enable terminal user interface.", default_value = "false")]
    tui: bool,

    #[arg(long, help = "TUI refresh rate in milliseconds.", default_value = "150")]
    refresh_rate: u64,

    #[arg(
        long,
        help = "Select the used algorithm by the agents.",
        default_value = "Tremeaux",
        value_parser = ["Tremeaux", "WallFollower", "Alian"]
    )]
    algorithm: String,
}

fn main() {
    let args = Args::parse();
    Logger::init(args.debug);
    let logger = Logger::get_instance();

    if args.offline {
        logger.info("Running in offline mode.");
        return;
    }

    let config =
        ClientConfig { server_addr: format!("{}:{}", args.host, args.port), team_name: args.team };
    let client = GameClient::new(config);

    if args.tui {
        let mut tui = tui::Tui::new(args.refresh_rate).expect("Failed to initialize TUI.");
        let tui_state = tui.get_state();

        if let Ok(mut state) = tui_state.lock() {
            for i in 0..args.players {
                state.register_agent(format!("Player{}", i + 1));
            }
        }

        tui.enter().expect("Failed to enter TUI.");
        let tui_handle = std::thread::spawn(move || {
            if let Err(err) = tui.run() {
                logger.error(&format!("TUI error: {}", err));
                std::process::exit(1);
            }
        });

        if let Err(e) = client.run(args.retries, args.players, Some(tui_state), args.algorithm) {
            e.log_error(logger);
            std::process::exit(1);
        }

        if let Err(e) = tui_handle.join() {
            logger.error(&format!("TUI thread error: {:?}", e));
        }
    } else {
        if let Err(e) = client.run(args.retries, args.players, None, args.algorithm) {
            e.log_error(logger);
            std::process::exit(1);
        }

        logger.info("All agents have finished their tasks.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = Args::parse_from(["test"]);
        assert_eq!(args.host, "localhost");
        assert_eq!(args.port, 8778);
        assert_eq!(args.retries, 5);
        assert_eq!(args.team, "Groupe1");
        assert_eq!(args.players, 3);
        assert!(!args.offline);
        assert!(!args.debug);
        assert!(!args.tui);
        assert_eq!(args.refresh_rate, 150);
    }

    #[test]
    fn test_invalid_port() {
        let result = Args::try_parse_from(["test", "--port", "1023"].iter());
        assert!(result.is_err());

        let result = Args::try_parse_from(["test", "--port", "65536"].iter());
        assert!(result.is_err());

        let result = Args::try_parse_from(["test", "--port", "invalid"].iter());
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_port_range() {
        let args = Args::parse_from(["test", "--port", "1024"].iter());
        assert_eq!(args.port, 1024);

        let args = Args::parse_from(["test", "--port", "65535"].iter());
        assert_eq!(args.port, 65535);

        let args = Args::parse_from(["test", "--port", "8080"].iter());
        assert_eq!(args.port, 8080);
    }

    #[test]
    fn test_client_config() {
        let args = Args::parse_from(
            ["test", "--host-address", "example.com", "--port", "9000", "--team", "TeamB"].iter(),
        );

        let config = ClientConfig {
            server_addr: format!("{}:{}", args.host, args.port),
            team_name: args.team,
        };

        assert_eq!(config.server_addr, "example.com:9000");
        assert_eq!(config.team_name, "TeamB");
    }
}
