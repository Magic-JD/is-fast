use crate::cli::command::{LogArgs, LogLevel};
use crate::config::files::log_path;
use chrono::Local;
use env_logger::Builder;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{env, thread};

pub fn init_logger(log_args: LogArgs) {
    let rust_log_level = env::var("RUST_LOG");
    if let Ok(level) = rust_log_level {
        log_to_file(&level);
    } else if log_args.log {
        let log_level = match log_args.log_level.unwrap_or_default() {
            LogLevel::Error => "is_fast=error",
            LogLevel::Warn => "is_fast=warn",
            LogLevel::Info => "is_fast=info",
            LogLevel::Debug => "is_fast=debug",
            LogLevel::Trace => "is_fast=trace",
        };
        log_to_file(log_level);
    }
}

fn log_to_file(level: &str) {
    let log_file_path = log_path();
    let log_file = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_path.as_path())
            .expect("Failed to open log file"),
    ));

    let log_file_clone = Arc::clone(&log_file);
    Builder::new()
        .parse_filters(level)
        .format(move |_, record| {
            let timestamp = Local::now().format("%y-%m-%d %H:%M:%S%.6f").to_string();
            let thread_id = thread::current().id();
            if let Ok(mut file) = log_file_clone.lock() {
                writeln!(
                    file,
                    "[{}] [{}] [{}] [{:?}] {}",
                    timestamp,
                    record.level(),
                    record.target(),
                    thread_id,
                    record.args()
                )
                .ok();
            }
            Ok(())
        })
        .init();
    // Log to error so it's not piped.
    eprintln!(
        "Logging activated (Level {level}). Logs located at {}",
        log_file_path.display()
    );
}
