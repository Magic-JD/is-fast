use std::fs;
use std::path::PathBuf;
use crate::config::constants::DEFAULT_CONFIG_LOCATION;

pub fn write_default_to_user(config_path: &PathBuf) -> Result<(), String> {
    fs::write(config_path, DEFAULT_CONFIG_LOCATION)
        .map_err(|e| format!("Error writing config file: {}", e))
}

