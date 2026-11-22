use clap::Parser;
use server::server::{GameServer, ServerConfig};
use shared::maze::GeneratorAlgorithm;
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

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

    #[arg(
        short,
        long,
        help = "Seed for the maze, the exit and the start cells. Random if not set."
    )]
    seed: Option<u64>,

    #[arg(long, help = "Maze size as width,height.", default_value = "50,50", value_parser = parse_size)]
    maze: (usize, usize),

    #[arg(long, help = "Players per team.", default_value = "3")]
    team_size: u8,

    #[arg(long, help = "Maze generator.", value_enum, default_value_t = Generator::RecursiveBacktracking)]
    generator: Generator,

    #[arg(long, help = "Enable debug logs.", default_value = "false")]
    debug: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Generator {
    /// Long winding corridors and deep dead ends, like the reference server's default.
    RecursiveBacktracking,
    /// Easier: an open top row that every cell can reach going up.
    Sidewinder,
}

impl From<Generator> for GeneratorAlgorithm {
    fn from(generator: Generator) -> Self {
        match generator {
            Generator::RecursiveBacktracking => Self::RecursiveBacktracker,
            Generator::Sidewinder => Self::Sidewinder,
        }
    }
}

fn parse_size(text: &str) -> Result<(usize, usize), String> {
    let (width, height) = text.split_once(',').ok_or("expected width,height")?;
    let parse = |n: &str| n.trim().parse::<usize>().ok().filter(|&n| n > 0);
    parse(width).zip(parse(height)).ok_or_else(|| "sizes must be positive numbers".to_string())
}

fn main() {
    let args = Args::parse();
    let seed = args.seed.unwrap_or_else(rand::random);
    let (width, height) = args.maze;
    let config = ServerConfig {
        host: args.host,
        port: args.port,
        seed,
        max_players_per_team: args.team_size,
        width,
        height,
        generator: args.generator.into(),
    };
    let level = if args.debug { LevelFilter::DEBUG } else { LevelFilter::INFO };
    tracing_subscriber::fmt().with_max_level(level).init();
    info!("Seed {seed}, pass --seed {seed} to replay this game.");

    let server = GameServer::new(config);
    if let Err(e) = server.run() {
        error!("{e}");
    }
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
    fn test_maze_size() {
        let args = Args::try_parse_from(["test", "--maze", "12,8"]).unwrap();
        assert_eq!(args.maze, (12, 8));
        assert!(Args::try_parse_from(["test", "--maze", "12"]).is_err());
        assert!(Args::try_parse_from(["test", "--maze", "0,8"]).is_err());
    }

    #[test]
    fn test_custom_port() {
        let args = Args::try_parse_from(["test", "--port", "8080"]).unwrap();
        assert_eq!(args.port, 8080);
        assert_eq!(args.host, "localhost");
    }
}
