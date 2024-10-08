use super::log_level::LogLevel;
use chrono::Local;
use core::fmt;
use std::ffi::OsStr;
use std::fmt::Write;
use std::path::Path;
use std::path::PathBuf;
use whoami::{self, fallible};

const DEFAULT_TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S%z";
const DEFAULT_MESSAGE_FORMAT: &str =
    "{TIMESTAMP} | {EXE_NAME} | {SYSTEM_NAME} | {USER_NAME} | {LEVEL} | {MESSAGE}";

#[derive(Debug)]
pub enum ConfigError {
    InvalidFormat(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::InvalidFormat(details) => write!(f, "Invalid format: {}", details),
        }
    }
}

impl std::error::Error for ConfigError {}

#[derive(Clone, Debug)]
pub struct LoggerConfiguration {
    log_dir: PathBuf,                   // root directory of log folders
    file_extension: String,             // extension of log file
    days_stored: Option<u32>,           // Number of days to keep
    executions_stored: Option<u32>,     // Number of executions/folders to keep
    filter_log_level: Option<LogLevel>, // Lowest severity that will be show
    exe_name: String,                   // Name of the executable
    system_name: String,                // Name of the system
    user_name: String,                  // Name of the user (with domain if present)
    message_format: Option<String>,     // Format of message written to log file
    timestamp_format: Option<String>,   // Format of timestamp if present in message_format
}

impl LoggerConfiguration {
    pub fn new(
        log_dir: PathBuf,
        file_extension: &str,
        days_stored: Option<u32>,
        executions_stored: Option<u32>,
        filter_log_level: Option<LogLevel>,
    ) -> Self {
        let exe_name = match std::env::current_exe()
            .ok()
            .as_ref()
            .map(Path::new)
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .map(String::from)
        {
            Some(x) => x,
            None => "Unknown".to_string(),
        };

        let system_name = match fallible::hostname() {
            Ok(x) => x,
            Err(_) => "Unknown".to_string(),
        };

        let user_domain = match std::env::var("USERDOMAIN") {
            Ok(val) => format!("{}\\", val),
            Err(_) => "".to_string(),
        };

        let user_name = match fallible::username() {
            Ok(x) => format!("{}{}", user_domain, x),
            Err(_) => "Unknown".to_string(),
        };

        LoggerConfiguration {
            log_dir,
            file_extension: file_extension.to_string(),
            days_stored,
            executions_stored,
            filter_log_level,
            exe_name,
            system_name,
            user_name,
            message_format: None,
            timestamp_format: None,
        }
    }

    /// Used to filter LogLevels that are logged
    /// Any LogLevel whose severity is lower than the `filter_level` will be ignored during logging
    /// The severity is a i32 value and higher values represent more severe information
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut config = LoggerConfiguration::default();
    /// config.set_filter_level(&LogLevel::Info); // The default LogLevels would be ignored: DEBUG, TRACE
    /// ```
    ///
    /// # Notes
    ///
    /// Severity can be checked using i32::from() on a &LogLevel variant
    /// ```rust
    /// let level = LogLevel::Error;
    /// let severity = i32::from(&level);
    /// println!("{} severity = {}", level, severity);
    /// ```
    ///
    pub fn set_filter_level(&mut self, filter_level: LogLevel) {
        self.filter_log_level = Some(filter_level);
    }

    /// Return the LogLevel used to filter log messages
    pub fn get_filter_level(&self) -> Option<LogLevel> {
        self.filter_log_level.clone()
    }

    /// Return a String with the message format
    pub fn get_message_format(&self) -> &str {
        if let Some(x) = &self.message_format {
            x
        } else {
            DEFAULT_MESSAGE_FORMAT
        }
    }

    /// Define the format of the message using keywords
    ///
    /// By default, the format will be:
    /// {TIMESTAMP} | {EXE_NAME} | {SYSTEM_NAME} | {USER_NAME} | {LEVEL} | {MESSAGE}
    ///
    /// Possible keywords (must have include {})
    /// {TIMESTAMP} - The format of the timestamp can be set with set_timestamp_format
    /// {EXE_NAME}
    /// {SYSTEM_NAME}
    /// {USER_NAME}
    /// {LEVEL}
    /// {MESSAGE}
    ///
    /// # Arguments
    ///
    /// * `format` - A string slice that defines the log format
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut config = LoggerConfiguration::default();
    /// config.set_message_format("{TIMESTAMP} | {LEVEL} | {MESSAGE}")
    /// ```
    ///
    /// # Notes
    ///
    /// Only {LEVEL} and {MESSAGE} are obligatory
    pub fn set_message_format(&mut self, format: &str) -> Result<(), ConfigError> {
        if !format.contains("{MESSAGE}") {
            eprintln!("Message format must contain {{MESSAGE}}. Message format is unchanged");
            Err(ConfigError::InvalidFormat(
                "Message format must contain {MESSAGE}".to_string(),
            ))
        } else if !format.contains("{LEVEL}") {
            eprintln!("Message format must contain {{LEVEL}}. Message format is unchanged");
            Err(ConfigError::InvalidFormat(
                "Message format must contain {LEVEL}".to_string(),
            ))
        } else {
            self.message_format = Some(format.to_string());
            Ok(())
        }
    }

    /// Return a String with the timestamp format
    pub fn get_timestamp_format(&self) -> &str {
        if let Some(x) = &self.timestamp_format {
            x
        } else {
            DEFAULT_TIMESTAMP_FORMAT
        }
    }

    /// Define the format of the timestamp {TIMESTAMP} shown in the log message
    ///
    /// By default, the timestamp_format is "%Y-%m-%d %H:%M:%S%z" and the time is in the local timezone
    ///
    /// # Arguments
    ///
    /// * `format` - A string slice that defines how the timestamp will be formatted (Time in local timezone)
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = LoggerConfiguration::default();
    /// config.set_timestamp_format("%Y-%m-%d %H:%M:%S%z")
    /// ```
    pub fn set_timestamp_format(&mut self, format: &str) -> Result<(), ConfigError> {
        // Use result to catch a panic when trying the format
        let mut formatted_time = String::new();
        let result = write!(formatted_time, "{}", Local::now().format(format));

        if result.is_err() {
            eprintln!("Invalid timestamp format. Timestamp format is unchanged");
            Err(ConfigError::InvalidFormat(
                "Invalid timestamp format".to_string(),
            ))
        } else {
            self.timestamp_format = Some(format.to_string());
            Ok(())
        }
    }

    pub fn get_system_name(&self) -> &str {
        &self.system_name
    }

    pub fn get_exe_name(&self) -> &str {
        &self.exe_name
    }

    pub fn get_user_name(&self) -> &str {
        &self.user_name
    }

    pub fn get_log_dir(&self) -> &Path {
        &self.log_dir
    }

    pub fn get_file_extension(&self) -> &str {
        &self.file_extension
    }

    pub fn get_days_stored(&self) -> Option<u32> {
        self.days_stored
    }

    pub fn get_executions_stored(&self) -> Option<u32> {
        self.executions_stored
    }
}

impl Default for LoggerConfiguration {
    // Implementing the Default trait to provide default values
    fn default() -> Self {
        LoggerConfiguration::new(
            PathBuf::from("./logs"), // Default log directory
            "txt",                   // Default file extension
            None,                    // Default is no limit on days stored
            None,                    // Default is no limit on executions stored
            None,                    // All log levels will be shown
        )
    }
}
