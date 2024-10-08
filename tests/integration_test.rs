use std::fs;
use std::time::Duration;
use std::{path::PathBuf, thread::sleep};

use exec_logger::log_level::LogLevel;
use exec_logger::{config, log, log_level};

#[test]
fn test_config() {
    let config = config::LoggerConfiguration::new(
        PathBuf::from("test_files"),
        "LOG",
        Some(7),
        Some(5),
        Some(log_level::LogLevel::Info),
    );
    println!("Config created = {:?}", config);
}

#[test]
fn test_log() {
    let config = config::LoggerConfiguration::new(
        PathBuf::from("test_files"),
        "LOG",
        Some(7),
        Some(5),
        Some(log_level::LogLevel::Info),
    );
    println!("Config created = {:?}", config);

    // Try to send an INFO message before initializing
    log::info("Message 1");

    log::initialize(config);

    // Try to send an INFO after initializing
    log::info("Message 2");

    // Try to send a ERROR
    log::error("Message 3");

    // Try to send a TRACE message
    log::trace("Message 4");

    // Get the log file path
    let log_file_path = log::get_log_file_path().unwrap();
    println!("Log file path = {:?}", log_file_path);

    // Read file path
    let contents = fs::read_to_string(log_file_path).unwrap();

    // Check if messages are in the log file
    assert!(
        !contents.contains("Message 1"),
        "Message 1 should not be present"
    );
    assert!(
        contents.contains("Message 2"),
        "Message 2 should be present"
    );
    assert!(
        contents.contains("Message 3"),
        "Message 3 should be present"
    );
    assert!(
        !contents.contains("Message 4"),
        "Message 4 should not be present"
    );
}

#[test]
fn test_multiple_initialization() {
    for i in 1..6 {
        let mut config = config::LoggerConfiguration::new(
            PathBuf::from(format!("test_files/{}", i)),
            "txt",
            Some(7),
            Some(50),
            Some(log_level::LogLevel::Info),
        );

        let _ = config.set_message_format("{USER_NAME} | {LEVEL} | {MESSAGE} | {TIMESTAMP}");

        let _ = config.set_timestamp_format("%y-%m-%d");
        log::initialize(config);
        sleep(Duration::from_secs(1));
    }
}

#[test]
fn test_custom_level() {
    let stat = log::create_custom_level("STAT", 25);

    let config = config::LoggerConfiguration::new(
        PathBuf::from("test_files"),
        "LOG",
        Some(7),
        Some(5),
        None,
    );

    log::initialize(config);

    log::custom("This is a STAT message", &stat);

    log::info("This is an INFO message")
}

#[test]
fn test_level_severity() {
    let config = config::LoggerConfiguration::new(
        PathBuf::from("test_files"),
        "LOG",
        Some(7),
        Some(5),
        None,
    );

    let custom1 = log::create_custom_level("CUSTOM1", 45);

    log::initialize(config);

    log::custom("This is a custom message", &custom1);

    log::info("This is an INFO message");

    log::debug("This is a debug message");

    log::info("Logging all default severity levels");

    log::trace(&format!("TRACE = {}", i32::from(&LogLevel::Trace)));
    log::debug(&format!("DEBUG = {}", i32::from(&LogLevel::Debug)));
    log::info(&format!("INFO = {}", i32::from(&LogLevel::Info)));
    log::warn(&format!("WARN = {}", i32::from(&LogLevel::Warn)));
    log::error(&format!("ERROR = {}", i32::from(&LogLevel::Error)));
    log::custom(&format!("CUSTOM1 = {}", i32::from(&custom1)), &custom1);
}
