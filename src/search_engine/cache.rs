use crate::config::load::Config;
use crate::errors::error::IsError;
use bincode;
use bincode::{Decode, Encode};
use dirs::data_dir;
use once_cell::sync::Lazy;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

static HTML_CACHE: Lazy<Cache> = Lazy::new(Cache::new);

#[derive(Debug, Deserialize, Clone)]
pub enum CacheMode {
    Read,
    Write,
    ReadWrite,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    cache_mode: CacheMode,
    max_size: usize,
    ttl: i64,
    cull: u8,
}

impl CacheConfig {
    pub fn new(cache_mode: CacheMode, max_size: usize, ttl: i64, cull: u8) -> Self {
        Self {
            cache_mode,
            max_size,
            ttl,
            cull,
        }
    }
}

struct Cache {
    connection: Arc<Mutex<Connection>>,
    config: CacheConfig,
}

#[derive(Encode, Decode, Debug)]
struct HtmlCached {
    html: String,
    timestamp: i64,
}

impl Cache {
    pub fn new() -> Self {
        let conn = Connection::open(get_cache_path()).expect("Failed to open database");
        let config = Config::get_cache_config();
        let cache = Cache {
            connection: Arc::new(Mutex::new(conn)),
            config,
        };
        cache.init_db().expect("Failed to initialize database");
        cache
    }

    fn init_db(&self) -> Result<(), IsError> {
        self.get_connection().execute(
            "CREATE TABLE IF NOT EXISTS cache (
                url TEXT PRIMARY KEY,
                html TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert(&self, key: &str, value: &str) -> Result<(), IsError> {
        match self.config.cache_mode {
            CacheMode::Read | CacheMode::Disabled => return Ok(()),
            _ => {}
        }
        let timestamp = Self::current_time()? + self.config.ttl;
        let cache_size = self.get_cache_size()?;
        if cache_size >= self.config.max_size {
            self.purge_cache()?;
        }
        self.get_connection().execute(
            "INSERT INTO cache (url, html, timestamp) VALUES (?, ?, ?)
             ON CONFLICT(url) DO UPDATE SET html = excluded.html, timestamp = excluded.timestamp",
            params![key, value, timestamp],
        )?;
        Ok(())
    }

    fn get_cache_size(&self) -> Result<usize, IsError> {
        let count: usize = self
            .get_connection()
            .prepare("SELECT COUNT(*) FROM cache")?
            .query_row([], |row| row.get(0))?;
        Ok(count)
    }

    fn purge_cache(&self) -> Result<(), IsError> {
        let elements_to_retain =
            (self.config.max_size * (100 - self.config.cull as usize) + 99) / 100;
        self.get_connection()
            .prepare(
                "DELETE FROM cache WHERE url IN (
                SELECT url FROM cache ORDER BY timestamp ASC LIMIT (
                    SELECT COUNT(*) FROM cache) - ?
            )",
            )?
            .execute(params![elements_to_retain])?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, IsError> {
        match self.config.cache_mode {
            CacheMode::Write | CacheMode::Disabled => return Ok(None),
            _ => {}
        }

        let values = self.retrieve_value(key)?;

        if let Some((html, timestamp)) = values {
            if timestamp < Self::current_time()? {
                self.remove(key)?;
                return Ok(None);
            }
            return Ok(Some(html));
        }

        Ok(None)
    }

    fn retrieve_value(&self, key: &str) -> Result<Option<(String, i64)>, IsError> {
        let connection = self.get_connection();
        let mut stmt = connection.prepare("SELECT html, timestamp FROM cache WHERE url = ?")?;
        let mut rows = stmt.query(params![key])?;
        match rows.next()? {
            None => Ok(None),
            Some(row) => {
                let html: String = row.get(0)?;
                let timestamp: i64 = row.get(1)?;
                Ok(Some((html, timestamp)))
            }
        }
    }

    pub fn remove(&self, key: &str) -> Result<(), IsError> {
        let connection = self.get_connection();
        connection.execute("DELETE FROM cache WHERE url = ?", params![key])?;
        drop(connection);
        Ok(())
    }

    pub fn clear(&self) -> Result<(), IsError> {
        self.get_connection().execute("DELETE FROM cache", [])?;
        Ok(())
    }

    fn current_time() -> Result<i64, IsError> {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64)
    }

    fn get_connection(&self) -> MutexGuard<Connection> {
        self.connection
            .lock()
            .expect("Failed to get database connection")
    }
}

pub fn cached_pages_write(url: &str, html: &str) {
    HTML_CACHE
        .insert(url, html)
        .unwrap_or_else(|e| log::error!("Error when writing page to cache: {:?}", e));
}

pub fn cached_pages_read(url: &str) -> Option<String> {
    HTML_CACHE.get(url).unwrap_or_else(|e| {
        log::error!("Error when reading page from cache: {:?}", e);
        None
    })
}

pub fn clear() {
    HTML_CACHE
        .clear()
        .unwrap_or_else(|e| log::error!("Error when clearing cache: {:?}", e));
}

fn get_cache_path() -> PathBuf {
    let mut path = data_dir().expect("Failed to determine data directory");
    path.push("is-fast");
    fs::create_dir_all(&path).expect("Failed to create data directory");
    path.push("is-fast.db");
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    const MS_IN_SECOND: i64 = 1000;

    #[test]
    #[serial]
    fn test_cache_can_be_read_and_written_to() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        env::set_var("XDG_DATA_HOME", path);
        let _ = Cache::new(); // Just to init db in temp dir
        let path = get_cache_path();
        let cache = Connection::open(path).expect("Failed to open cache");
        let cache = Cache {
            connection: Arc::new(Mutex::new(cache)),
            config: CacheConfig::new(CacheMode::ReadWrite, 2, MS_IN_SECOND * 5, 50),
        };
        for i in 1..=5 {
            cache
                .insert(
                    format!("http://localhost:8080/{i}").as_str(),
                    format!("html{i}").as_str(),
                )
                .unwrap();
        }
        assert!(cache.get("http://localhost:8080/1").unwrap().is_none());
        assert!(cache.get("http://localhost:8080/2").unwrap().is_none());
        assert!(cache.get("http://localhost:8080/3").unwrap().is_none());
        assert_eq!(
            cache.get("http://localhost:8080/4").unwrap(),
            Some(String::from("html4"))
        );
        assert_eq!(
            cache.get("http://localhost:8080/5").unwrap(),
            Some(String::from("html5"))
        );
    }

    #[test]
    #[serial]
    fn test_cache_ttl_removes_expired() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        env::set_var("XDG_DATA_HOME", path);

        let _ = Cache::new(); // Just to init db in temp dir
        let path = get_cache_path();
        let cache = Connection::open(path).expect("Failed to open cache");
        let cache = Cache {
            connection: Arc::new(Mutex::new(cache)),
            config: CacheConfig::new(CacheMode::ReadWrite, 2, 0, 50),
        };
        for i in 1..=5 {
            cache
                .insert(
                    format!("http://localhost:8080/{i}").as_str(),
                    format!("html{i}").as_str(),
                )
                .unwrap();
        }
        assert!(cache.get("http://localhost:8080/1").unwrap().is_none());
        assert!(cache.get("http://localhost:8080/2").unwrap().is_none());
        assert!(cache.get("http://localhost:8080/3").unwrap().is_none());
        assert!(cache.get("http://localhost:8080/4").unwrap().is_none());
        assert!(cache.get("http://localhost:8080/5").unwrap().is_none());
    }
}
