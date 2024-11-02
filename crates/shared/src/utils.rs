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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_error() {
        print_error("Error message");
    }

    #[test]
    fn test_print_log() {
        print_log("Log message", Color::Green);
    }
}
