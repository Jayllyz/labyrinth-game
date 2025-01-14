use crate::utils::{Color, ColorsAnsi};
use std::sync::OnceLock;

pub static INSTANCE: OnceLock<Logger> = OnceLock::new();

#[derive(Clone)]
pub enum LogLevel {
    Info,
    Debug,
    Warning,
    Error,
}

pub struct Logger {
    debug_enabled: bool,
}

impl Logger {
    pub fn init(debug_enabled: bool) {
        let _ = INSTANCE.set(Logger { debug_enabled });
    }

    pub fn get_instance() -> &'static Logger {
        INSTANCE.get().expect("Logger must be initialized before use.")
    }

    pub fn is_debug_enabled(&self) -> bool {
        self.debug_enabled
    }

    pub fn debug(&self, msg: &str) {
        if self.debug_enabled {
            self.log(msg, "DEBUG", Color::Blue);
        }
    }

    pub fn info(&self, msg: &str) {
        self.log(msg, "INFO", Color::Green);
    }

    pub fn warn(&self, msg: &str) {
        self.log(msg, "WARN", Color::Orange);
    }

    pub fn error(&self, msg: &str) {
        self.log_error(msg);
    }

    fn log(&self, msg: &str, level: &str, color: Color) {
        let color_code = match color {
            Color::Red => ColorsAnsi::RED,
            Color::Orange => ColorsAnsi::ORANGE,
            Color::Blue => ColorsAnsi::BLUE,
            Color::Green => ColorsAnsi::GREEN,
            Color::Reset => ColorsAnsi::RESET,
        };
        println!("{}[{}]{} - {}", color_code, level, ColorsAnsi::RESET, msg);
    }

    fn log_error(&self, msg: &str) {
        eprintln!("{}[ERROR]{} - {}", ColorsAnsi::RED, ColorsAnsi::RESET, msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger() {
        Logger::init(true);
        let logger = Logger::get_instance();
        logger.debug("Debug message");
        logger.info("Info message");
        logger.warn("Warning message");
        logger.error("Error message");
        if logger.is_debug_enabled() {
            assert!(logger.is_debug_enabled());
        }
    }
}
