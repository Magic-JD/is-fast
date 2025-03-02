use crate::config::write::write_default_to_user;
use std::fs;

pub fn run() {
    println!("Generating config file...");
    let config_directory = dirs::config_dir();
    match config_directory {
        Some(config_dir) => {
            let is_fast_dir = config_dir.join("is-fast");
            let config_path = is_fast_dir.join("config.toml");

            fs::create_dir_all(&is_fast_dir)
                .map_err(|e| format!("Error creating config directory: {}", e))
                .and_then(|_| {
                    if !config_path.exists() {
                        write_default_to_user(&config_path)
                    } else {
                        Err(format!("Config file already exists at {:?}", config_path))
                    }
                })
                .map(|_| println!("Config file generated at {:?}", config_path))
                .unwrap_or_else(|e| eprintln!("{}", e));
        }
        None => {
            println!("Could not determine config directory");
        }
    }
}
