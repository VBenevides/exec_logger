use core::fmt;
use std::cmp::Ordering;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Custom(i32, String), // Holds a custom log level
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Custom(_, display) => write!(f, "{}", display),
        }
    }
}

impl From<&LogLevel> for i32 {
    fn from(log_level: &LogLevel) -> i32 {
        match log_level {
            LogLevel::Error => 50,
            LogLevel::Warn => 40,
            LogLevel::Info => 30,
            LogLevel::Debug => 20,
            LogLevel::Trace => 10,
            LogLevel::Custom(value, _) => *value,
        }
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_value: i32 = self.into();
        let other_value: i32 = other.into();

        self_value.cmp(&other_value)
    }
}
