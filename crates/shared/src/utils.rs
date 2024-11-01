use std::env;

pub enum Color {
    Red,
    Blue,
    Green,
    Reset,
}

pub struct ColorsAnsi;

impl ColorsAnsi {
    pub const RED: &'static str = "\x1b[31m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const RESET: &'static str = "\x1b[0m";
}

pub fn print_error(msg: &str) {
    eprintln!("{}{}{}\n", ColorsAnsi::RED, msg, ColorsAnsi::RESET);
}

pub fn print_log(msg: &str, color: Color) {
    let color = match color {
        Color::Red => ColorsAnsi::RED,
        Color::Blue => ColorsAnsi::BLUE,
        Color::Green => ColorsAnsi::GREEN,
        Color::Reset => ColorsAnsi::RESET,
    };
    println!("{}{}{}\n", color, msg, ColorsAnsi::RESET);
}

pub fn get_server_address() -> String {
    const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:7878";
    env::args()
        .nth(1)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_SERVER_ADDRESS.to_string())
}

pub fn get_player_name() -> String {
    const DEFAULT_PLAYER_NAME: &str = "Player1";
    env::args().nth(2).filter(|s| !s.is_empty()).unwrap_or_else(|| DEFAULT_PLAYER_NAME.to_string())
}

pub fn get_team_name() -> String {
    const DEFAULT_TEAM_NAME: &str = "Team1";
    env::args().nth(3).filter(|s| !s.is_empty()).unwrap_or_else(|| DEFAULT_TEAM_NAME.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_server_address() {
        let address = get_server_address();
        const DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:7878";
        assert_eq!(address, DEFAULT_SERVER_ADDRESS);
    }

    #[test]
    fn test_get_player_name() {
        let name = get_player_name();
        const DEFAULT_PLAYER_NAME: &str = "Player1";
        assert_eq!(name, DEFAULT_PLAYER_NAME);
    }

    #[test]
    fn test_get_team_name() {
        let name = get_team_name();
        const DEFAULT_TEAM_NAME: &str = "Team1";
        assert_eq!(name, DEFAULT_TEAM_NAME);
    }

    #[test]
    fn test_print_error() {
        print_error("Error message");
    }

    #[test]
    fn test_print_log() {
        print_log("Log message", Color::Green);
    }
}
