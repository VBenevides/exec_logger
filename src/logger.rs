use super::config::LoggerConfiguration;
use super::log_level::LogLevel;
use chrono::{Duration, Local, NaiveDateTime};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Logger {
    config: LoggerConfiguration,
    log_file_path: PathBuf,
}

impl Logger {
    pub fn new(config: LoggerConfiguration) -> Result<Self, std::io::Error> {
        let mut logger = Logger {
            config,
            log_file_path: PathBuf::new(),
        };

        let _ = &logger.delete_old_logs()?;

        let _ = &logger.create_current_log()?;

        logger.info("Logger initialized");

        Ok(logger)
    }

    /// Get log file path
    pub fn get_log_file_path(&self) -> PathBuf {
        self.log_file_path.clone()
    }

    /// List folders in a path
    fn list_folders(directory_path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut folders = Vec::new();

        // Read the directory entries using std::fs::read_dir
        if let Ok(entries) = std::fs::read_dir(&directory_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        folders.push(path);
                    }
                }
            }
        }

        Ok(folders)
    }

    /// Delete old logs according to the configuration file
    /// Old folders are deleted following 2 conditions in the LoggerConfiguration
    /// 1) Logs older than X days (days_stored)
    /// 2) Oldest logs exceeding the max number of logs (executions_stored)
    fn delete_old_logs(&self) -> Result<(), std::io::Error> {
        let log_dir_root = self.config.get_log_dir();
        let days_stored = self.config.get_days_stored();
        let executions_stored = self.config.get_executions_stored();

        // 1) Delete based on date
        if let Some(days_stored) = days_stored {
            let log_folders = Logger::list_folders(log_dir_root)?;
            let limit_datetime =
                Local::now().naive_local() - Duration::days(i64::from(days_stored));

            for folder in log_folders {
                let folder_name = folder.file_name();
                if folder_name.is_none() {
                    continue;
                }

                // folder_name.unwrap() is safe, because folder_name will not be None
                let folder_name = folder_name.unwrap().to_str();
                if folder_name.is_none() {
                    continue;
                }

                let folder_name = folder_name.unwrap();

                let datetime = NaiveDateTime::parse_from_str(folder_name, "%Y-%m-%d %H_%M_%S");
                if datetime.is_err() {
                    // If it is not possible to get a datetime from the file name, then the name was not created by the logger and must
                    // not be deleted by the logger. It should be manually deleted
                    continue;
                }

                let datetime = datetime.unwrap();

                // I don't want to raise an error if it is not possible to delete the folder, because this may happen if
                // the program is being executed with a different permission from a previous execution
                // In this case, old logs need to be manually deleted
                if datetime < limit_datetime {
                    if let Err(e) = std::fs::remove_dir_all(&folder) {
                        eprintln!("Failed to delete old log folder {:?}: {}", folder, e);
                    }
                }
            }
        } else {
            #[cfg(not(debug_assertions))]
            {
                println!("Logger not configured to delete older executions based on date");
            }
        }

        // 2) Delete based on number of logs
        if let Some(executions_stored) = executions_stored {
            let mut log_folders = Logger::list_folders(log_dir_root)?;

            // If the executions to store is X, we must delete X - 1 (remove 1 to accomodate the current execution)
            let mut num_delete = log_folders.len() as i64 - (executions_stored as i64 - 1);

            if num_delete > 0 {
                // sort file names to get the oldest one first, this works because the file names are in ISO 8601
                log_folders.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));

                for folder in log_folders {
                    let folder_name = folder.file_name();
                    if folder_name.is_none() {
                        continue;
                    }

                    // folder_name.unwrap() is safe, because folder_name will not be None
                    let folder_name = folder_name.unwrap().to_str();
                    if folder_name.is_none() {
                        continue;
                    }

                    let folder_name = folder_name.unwrap();

                    let datetime = NaiveDateTime::parse_from_str(folder_name, "%Y-%m-%d %H_%M_%S");
                    if datetime.is_err() {
                        // If it is not possible to get a datetime from the file name, then the name was not created by the logger and must
                        // not be deleted by the logger. It should be manually deleted
                        continue;
                    }

                    // try to delete the folder
                    if let Err(e) = std::fs::remove_dir_all(&folder) {
                        eprintln!("Failed to delete old log folder {:?}: {}", folder, e);
                    } else {
                        num_delete -= 1;
                    }

                    if num_delete <= 0 {
                        break;
                    }
                }
            }
        } else {
            #[cfg(not(debug_assertions))]
            {
                println!(
                    "Logger not configured to delete olders executions based on the number of executions"
                )
            }
        }

        Ok(())
    }

    /// Create current log file
    fn create_current_log(&mut self) -> Result<(), std::io::Error> {
        let current_datetime: NaiveDateTime = Local::now().naive_local();
        let folder_name = current_datetime.format("%Y-%m-%d %H_%M_%S").to_string();
        let log_dir = self.config.get_log_dir().join(PathBuf::from(folder_name));
        std::fs::create_dir_all(&log_dir)?;

        let file_extension = self.config.get_file_extension();
        let file_name = format!("execution_log.{}", file_extension);
        let log_file_path = log_dir.join(PathBuf::from(file_name));

        self.log_file_path = log_file_path;
        Ok(())
    }

    /// Create the log message from the format
    fn format_message(&self, message: &str, level: &LogLevel) -> String {
        // Technically, using a HashMap could be cleaner instead of using many contains
        // but the idea is to evaluate the parts of the message only if necessary
        let mut msg = self.config.get_message_format().to_string();

        if msg.contains("{TIMESTAMP}") {
            // Gets the time only if necessary
            let timestamp_format = self.config.get_timestamp_format();
            let now = Local::now().format(timestamp_format);
            msg = msg.replace("{TIMESTAMP}", &now.to_string());
        }

        if msg.contains("{EXE_NAME}") {
            msg = msg.replace("{EXE_NAME}", self.config.get_exe_name());
        }

        if msg.contains("{SYSTEM_NAME}") {
            msg = msg.replace("{SYSTEM_NAME}", self.config.get_system_name());
        }

        if msg.contains("{USER_NAME}") {
            msg = msg.replace("{USER_NAME}", self.config.get_user_name());
        }

        if msg.contains("{LEVEL}") {
            msg = msg.replace("{LEVEL}", &format!("{:<7}", level.to_string()));
        }

        if msg.contains("{MESSAGE}") {
            msg = msg.replace("{MESSAGE}", message);
        }

        if !msg.ends_with('\n') {
            msg.push('\n');
        }

        msg
    }

    /// Write the log message to stdout and to the log file
    fn log(&self, message: &str, level: &LogLevel) {
        // Check if the message level has severity higher than the minimum
        match self.config.get_filter_level() {
            Some(filter_level) => {
                if *level < filter_level {
                    return; // return from the function without doing anything
                }
            }
            None => (),
        }

        let message_formatted = &self.format_message(message, level);

        // Print to stdout
        print!("{}", message_formatted);

        // Open/create log file
        let log_file_res = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.log_file_path);

        if let Ok(mut log_file) = log_file_res {
            let write_result = log_file.write_all(message_formatted.as_bytes());

            if let Err(e) = write_result {
                eprintln!("Unable to write log message to log file: {}", e);
            }
        } else if let Err(e) = log_file_res {
            eprintln!("Unable to open log file: {}", e)
        }
    }

    /// Send message of type INFO
    pub fn info(&self, message: &str) {
        self.log(message, &LogLevel::Info);
    }

    /// Send message of type ERROR
    pub fn error(&self, message: &str) {
        self.log(message, &LogLevel::Error);
    }

    /// Send message of type WARN
    pub fn warn(&self, message: &str) {
        self.log(message, &LogLevel::Warn);
    }

    /// Send message of type DEBUG
    pub fn debug(&self, message: &str) {
        self.log(message, &LogLevel::Debug);
    }

    /// Send message of type TRACE
    pub fn trace(&self, message: &str) {
        self.log(message, &LogLevel::Trace);
    }

    /// Send message of type CUSTOM (Defined by user)
    pub fn custom(&self, message: &str, level: &LogLevel) {
        self.log(message, level);
    }
}
