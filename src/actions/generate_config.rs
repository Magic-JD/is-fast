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
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_run_creates_config_file() {
        use std::env;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fake_home = temp_dir.path();

        env::set_var("XDG_CONFIG_HOME", fake_home);
        run();

        let config_path = fake_home.join("is-fast/config.toml");
        assert!(config_path.exists(), "Config file should be created");
    }

    #[test]
    fn test_run_fails_if_config_exists() {
        use std::env;
        use std::fs;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fake_home = temp_dir.path();

        env::set_var("XDG_CONFIG_HOME", fake_home);

        let config_path = fake_home.join("is-fast/config.toml");
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(&config_path, "existing content").unwrap();

        let output = std::panic::catch_unwind(run);

        assert!(output.is_ok(), "Function should not panic");
    }
}
