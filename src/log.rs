pub use self::functions::{
    create_custom_level, custom, debug, error, get_log_file_path, info, initialize, trace, warn,
};

mod functions {
    use crate::config::LoggerConfiguration;
    use crate::log_level::LogLevel;
    use crate::logger::Logger;

    use arc_swap::{ArcSwap, Guard};
    use once_cell::sync::OnceCell;
    use std::path::PathBuf;
    use std::sync::Arc;
    static LOGGER: OnceCell<ArcSwap<Logger>> = OnceCell::new();

    // Get internal logger from LOGGER
    fn get_logger() -> Option<Guard<Arc<Logger>>> {
        match LOGGER.get() {
            Some(x) => Some(x.load()),
            None => None,
        }
    }

    // Set internal logger for LOGGER
    fn set_logger(logger: Logger) {
        if LOGGER.get().is_none() {
            let _ = LOGGER.set(ArcSwap::from_pointee(logger));
        } else {
            LOGGER.get().unwrap().store(Arc::new(logger));
        }
    }

    /// Initializes the logger with the given configuration
    pub fn initialize(config: LoggerConfiguration) -> Result<(), Box<dyn std::error::Error>> {
        // There are other log implementations that allow for the log to be initialized only once
        // However this is not the case here, I want to be able to initialise the log more than
        // once if necessary to change the log file. Processes that execute for long periods might
        // occupy a lot of disk space and in this case it is possible to initialize the log again to
        // generate a new log file.

        let logger = Logger::new(config)?;
        set_logger(logger);
        Ok(())
    }

    /// Get the current log file path from the LOGGER
    pub fn get_log_file_path() -> Option<PathBuf> {
        if let Some(logger) = get_logger() {
            Some(logger.get_log_file_path())
        } else {
            eprintln!("Logger not initialized");
            None
        }
    }

    /// Logs a INFO message
    pub fn info(message: &str) {
        if let Some(logger) = get_logger() {
            logger.info(message);
        } else {
            eprintln!("Logger not initialized")
        }
    }

    /// Logs a ERROR message
    pub fn error(message: &str) {
        if let Some(logger) = get_logger() {
            logger.error(message);
        } else {
            eprintln!("Logger not initialized")
        }
    }

    /// Logs a DEBUG message
    pub fn debug(message: &str) {
        if let Some(logger) = get_logger() {
            logger.debug(message);
        } else {
            eprintln!("Logger not initialized")
        }
    }

    /// Logs a TRACE message
    pub fn trace(message: &str) {
        if let Some(logger) = get_logger() {
            logger.trace(message);
        } else {
            eprintln!("Logger not initialized")
        }
    }

    /// Logs a WARN message
    pub fn warn(message: &str) {
        if let Some(logger) = get_logger() {
            logger.warn(message);
        } else {
            eprintln!("Logger not initialized")
        }
    }

    /// Define a new LogLevel variant used for custom levels
    pub fn create_custom_level(name: &str, severity: i32) -> LogLevel {
        let level = LogLevel::Custom(severity, name.to_string());
        level
    }

    /// Logs a message with a custom log level
    pub fn custom(message: &str, level: &LogLevel) {
        if let Some(logger) = get_logger() {
            logger.custom(message, level);
        } else {
            eprintln!("Logger not initialized")
        }
    }
}
