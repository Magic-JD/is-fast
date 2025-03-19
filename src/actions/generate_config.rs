use crate::config::files::config_path;
use crate::config::load::DEFAULT_CONFIG_LOCATION;
use std::fs;

pub fn run() {
    println!("Generating config file...");
    let config_path = config_path();
    if config_path.exists() {
        eprintln!("Config file already exists at {config_path:?}");
    } else if let Err(message) = fs::write(&config_path, DEFAULT_CONFIG_LOCATION)
        .map_err(|e| format!("Error writing config file: {e}"))
    {
        eprintln!("{message}");
    } else {
        println!("Config file generated at {config_path:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_run_creates_config_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let fake_home = convert_to_canon(temp_dir);

        env::set_var("XDG_CONFIG_HOME", &fake_home);
        run();

        let config_path = fake_home.join("is-fast/config.toml");
        assert!(config_path.exists(), "Config file should be created");
    }

    fn convert_to_canon(temp_dir: TempDir) -> PathBuf {
        if temp_dir.path().is_relative() {
            fs::canonicalize(temp_dir.path()).expect("Failed to canonicalize temp dir")
        } else {
            temp_dir.path().to_path_buf()
        }
    }

    #[test]
    #[serial]
    fn test_run_fails_if_config_exists() {
        use std::env;
        use std::fs;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fake_home = convert_to_canon(temp_dir);

        env::set_var("XDG_CONFIG_HOME", &fake_home);

        let config_path = fake_home.join("is-fast/config.toml");
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(&config_path, "existing content").unwrap();

        let output = std::panic::catch_unwind(run);

        assert!(output.is_ok(), "Function should not panic");
    }
}
