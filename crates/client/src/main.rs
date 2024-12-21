use clap::Parser;
use client::client::{ClientConfig, GameClient};
use client::tui;
use shared::logger::Logger;
use shared::radar;

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

    #[arg(long, help = "Enable debug mode.", default_value = "false")]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    Logger::init(args.debug);
    let logger = Logger::get_instance();

    let config = ClientConfig {
        server_addr: format!("{}:{}", args.host, args.port),
        team_name: args.team,
        token: args.token,
    };

    if args.offline {
        println!("Running in offline mode (no connection to the server)");
        println!("Not implemented yet, exiting...");
        let decoded = radar::decode_base64("giLbMjIad/apapa");
        let radar_view = match radar::extract_data(&decoded) {
            Ok(view) => view,
            Err(e) => {
                logger.error(&format!("Error decoding radar view: {}", e));
                std::process::exit(1);
            }
        };
        println!("{:?}", radar_view.horizontal);
        println!("{:?}", radar_view.vertical);
        println!("{:?}", radar_view.cells);
        return;
    }

    let client = GameClient::new(config);
    let agents_count = args.players.unwrap_or(3);

    let mut tui = tui::Tui::new().expect("Failed to create TUI");
    let tui_state = tui.get_state();

    let num_agents = args.players.unwrap_or(3);
    if let Ok(mut state) = tui_state.lock() {
        for i in 0..num_agents {
            state.register_agent(format!("Player{}", i + 1));
        }
    }

    // Start TUI in separate thread
    std::thread::spawn(move || {
        if let Err(err) = tui.run() {
            eprintln!("Error running TUI: {}", err);
        }
    });

    if let Err(e) = client.run(args.retries, agents_count, Some(tui_state)) {
        e.log_error(logger);
        std::process::exit(1);
    }

    logger.info("All agents have finished their tasks.");
}
