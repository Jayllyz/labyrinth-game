pub struct ColorsAnsi;

impl ColorsAnsi {
    pub const RED: &'static str = "\x1b[31m";
    pub const ORANGE: &'static str = "\x1b[33m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const RESET: &'static str = "\x1b[0m";
}

pub fn print_error(msg: &str) {
    eprintln!("{}{}{}\n", ColorsAnsi::RED, msg, ColorsAnsi::RESET);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_error() {
        print_error("Error message");
    }
}
