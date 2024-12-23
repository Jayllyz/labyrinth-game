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

    #[arg(long, help = "Number of players in the team.")]
    #[arg(help_heading = "PLAYER OPTIONS")]
    players: Option<u8>,

    #[arg(long, help = "Run the client in offline mode.")]
    offline: bool,

    #[arg(long, help = "Enable debug logs.", default_value = "false")]
    debug: bool,

    #[arg(long, help = "Enable terminal user interface.", default_value = "false")]
    tui: bool,

    #[arg(long, help = "TUI refresh rate in milliseconds.", default_value = "100")]
    refresh_rate: u64,
}

fn main() {
    let args = Args::parse();
    Logger::init(args.debug);
    let logger = Logger::get_instance();

    let config =
        ClientConfig { server_addr: format!("{}:{}", args.host, args.port), team_name: args.team };

    if args.offline {
        logger.info("Running in offline mode.");
        return;
    }

    let client = GameClient::new(config);
    let agents_count = args.players.unwrap_or(3);

    if args.tui {
        let mut tui = tui::Tui::new(args.refresh_rate).expect("Failed to initialize TUI.");
        let tui_state = tui.get_state();

        if let Ok(mut state) = tui_state.lock() {
            for i in 0..agents_count {
                state.register_agent(format!("Player{}", i + 1));
            }
        }

        let tui_handle = std::thread::spawn(move || {
            if let Err(err) = tui.run() {
                logger.error(&format!("TUI error: {}", err));
                std::process::exit(1);
            }
        });

        if let Err(e) = client.run(args.retries, agents_count, Some(tui_state)) {
            e.log_error(logger);
            std::process::exit(1);
        }

        if let Err(e) = tui_handle.join() {
            logger.error(&format!("TUI thread error: {:?}", e));
        }
    } else {
        if let Err(e) = client.run(args.retries, agents_count, None) {
            e.log_error(logger);
            std::process::exit(1);
        }

        logger.info("All agents have finished their tasks.");
    }
}
