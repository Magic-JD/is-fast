use env_logger::Builder;
use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};

pub fn init_logger() {
    let rust_log_level = env::var("RUST_LOG");
    if let Ok(level) = rust_log_level {
        log_to_file(&level);
    }
}

fn log_to_file(level: &str) {
    if let Some(config_directory) = dirs::config_dir() {
        let is_fast_dir = config_directory.join("is-fast");
        let log_file_path = is_fast_dir.join("is-fast.log");
        create_dir_all(is_fast_dir).expect("Couldn't create log directory");
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
                if let Ok(mut file) = log_file_clone.lock() {
                    writeln!(file, "[{}] {}", record.level(), record.args()).ok();
                }
                Ok(())
            })
            .init();
        println!(
            "Logging activated. Logs located at {}",
            log_file_path.display()
        );
    } else {
        println!("Couldn't find config directory");
    }
}
