use crate::cli::command::CacheMode;
use crate::config::files::database_path;
use crate::errors::error::IsError;
use crate::search_engine::link::HtmlSource;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use parking_lot::MutexGuard;
use rusqlite::{params, Connection};
use std::io::Cursor;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zstd::{decode_all, encode_all};

static HTML_CACHE: Lazy<Cache> = Lazy::new(Cache::new);
static VERSION: u16 = 0;

#[derive(Debug, Clone, Default)]
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
}

impl Cache {
    pub fn new() -> Self {
        let conn = Connection::open(database_path()).expect("Failed to open database");
        let cache = Cache {
            connection: Arc::new(Mutex::new(conn)),
        };
        cache.init_db().expect("Failed to initialize database");
        cache
    }

    fn init_db(&self) -> Result<(), IsError> {
        self.get_connection().execute(
            "CREATE TABLE IF NOT EXISTS cache (
                url TEXT PRIMARY KEY,
                html BLOB NOT NULL,
                timestamp INTEGER NOT NULL,
                version INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert(&self, key: &HtmlSource, value: &str) -> Result<(), IsError> {
        let binding = key.get_config();
        let cache_config = binding.get_cache();
        match cache_config.cache_mode {
            CacheMode::Read | CacheMode::Never => return Ok(()),
            _ => {}
        }
        let timestamp = Self::current_time()? + cache_config.ttl;
        let cache_size = self.get_cache_size()?;
        if cache_size >= cache_config.max_size {
            self.purge_cache(cache_config)?;
        }
        let compressed_html = encode_all(Cursor::new(value), 3)?;
        self.get_connection().execute(
            "INSERT INTO cache (url, html, timestamp, version) VALUES (?, ?, ?, ?)
             ON CONFLICT(url) DO UPDATE SET html = excluded.html, timestamp = excluded.timestamp",
            params![key.get_url(), compressed_html, timestamp, VERSION],
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

    fn purge_cache(&self, cache_config: &CacheConfig) -> Result<(), IsError> {
        let elements_to_retain =
            (cache_config.max_size * (100 - cache_config.cull as usize)).div_ceil(100);
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

    pub fn get(&self, key: &HtmlSource) -> Result<Option<String>, IsError> {
        match key.get_config().get_cache().cache_mode {
            CacheMode::Write | CacheMode::Never => return Ok(None),
            _ => {}
        }

        let values = self.retrieve_value(key.get_url())?;

        if let Some((html, timestamp)) = values {
            if timestamp <= Self::current_time()? {
                log::debug!("Expired cache item {}", key.get_url());
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
                let compressed_html: Vec<u8> = row.get(0)?;
                let timestamp: i64 = row.get(1)?;
                let html = String::from_utf8(decode_all(Cursor::new(compressed_html))?)?;
                Ok(Some((html, timestamp)))
            }
        }
    }

    pub fn remove(&self, key: &HtmlSource) -> Result<(), IsError> {
        if let CacheMode::Never | CacheMode::Read = key.get_config().get_cache().cache_mode {
            return Ok(());
        }
        let connection = self.get_connection();
        let url = key.get_url();
        connection.execute("DELETE FROM cache WHERE url = ?", params![url])?;
        drop(connection);
        log::debug!("Removing: {url} from cache");
        Ok(())
    }

    pub fn clear(&self) -> Result<(), IsError> {
        self.get_connection().execute("DROP TABLE cache", [])?;
        Ok(())
    }

    fn current_time() -> Result<i64, IsError> {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64)
    }

    fn get_connection(&self) -> MutexGuard<Connection> {
        self.connection
            .try_lock_for(Duration::from_millis(100))
            .expect("Failed to get database connection")
    }
}

pub fn cached_pages_write(url: &HtmlSource, html: &str) {
    log::debug!("Caching {}", url.get_url());
    HTML_CACHE
        .insert(url, html)
        .unwrap_or_else(|e| log::error!("Error when writing page to cache: {:?}", e));
}

pub fn cached_pages_read(html_source: &HtmlSource) -> Option<String> {
    let cache_result = HTML_CACHE.get(html_source).unwrap_or_else(|e| {
        log::error!("Error when reading page from cache: {:?}", e);
        None
    });
    let url = html_source.get_url();
    match cache_result {
        Some(_) => log::debug!("Cache hit for {url}"),
        None => log::debug!("Cache miss for {url}"),
    };
    cache_result
}

pub fn cached_pages_purge(url: &HtmlSource) {
    HTML_CACHE.remove(url).unwrap_or_else(|e| {
        log::error!("Error when removing page from cache: {:?}", e);
    });
}

pub fn clear() {
    HTML_CACHE
        .clear()
        .unwrap_or_else(|e| log::error!("Error when clearing cache: {:?}", e));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search_engine::link::tests::TEST_CONFIG;
    use crate::search_engine::link::HtmlSource::LinkSource;
    use crate::search_engine::link::Link;
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
        TEST_CONFIG.write().cache = CacheConfig::new(CacheMode::ReadWrite, 2, MS_IN_SECOND * 5, 50);
        let path = database_path();
        let cache = Connection::open(path).expect("Failed to open cache");
        let cache = Cache {
            connection: Arc::new(Mutex::new(cache)),
        };
        for i in 1..=5 {
            cache
                .insert(
                    &LinkSource(Link::new(format!("http://localhost:8080/{i}").as_str())),
                    format!("html{i}").as_str(),
                )
                .unwrap();
        }
        assert!(cache
            .get(&LinkSource(Link::new("http://localhost:8080/1")))
            .unwrap()
            .is_none());
        assert!(cache
            .get(&LinkSource(Link::new("http://localhost:8080/2")))
            .unwrap()
            .is_none());
        assert!(cache
            .get(&LinkSource(Link::new("http://localhost:8080/3")))
            .unwrap()
            .is_none());
        assert_eq!(
            cache
                .get(&LinkSource(Link::new("http://localhost:8080/4")))
                .unwrap(),
            Some(String::from("html4"))
        );
        assert_eq!(
            cache
                .get(&LinkSource(Link::new("http://localhost:8080/5")))
                .unwrap(),
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
        TEST_CONFIG.write().cache = CacheConfig::new(CacheMode::ReadWrite, 2, 0, 50);
        let path = database_path();
        let cache = Connection::open(path).expect("Failed to open cache");
        let cache = Cache {
            connection: Arc::new(Mutex::new(cache)),
        };
        for i in 1..=5 {
            cache
                .insert(
                    &LinkSource(Link::new(format!("http://localhost:8080/{i}").as_str())),
                    format!("html{i}").as_str(),
                )
                .unwrap();
        }
        assert!(cache
            .get(&LinkSource(Link::new("http://localhost:8080/1")))
            .unwrap()
            .is_none());
        assert!(cache
            .get(&LinkSource(Link::new("http://localhost:8080/2")))
            .unwrap()
            .is_none());
        assert!(cache
            .get(&LinkSource(Link::new("http://localhost:8080/3")))
            .unwrap()
            .is_none());
        assert!(cache
            .get(&LinkSource(Link::new("http://localhost:8080/4")))
            .unwrap()
            .is_none());
        assert!(cache
            .get(&LinkSource(Link::new("http://localhost:8080/5")))
            .unwrap()
            .is_none());
    }
}
