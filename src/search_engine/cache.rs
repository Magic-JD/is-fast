use crate::config::load::Config;
use crate::errors::error::IsError;
use bincode;
use bincode::config::standard;
use bincode::{Decode, Encode};
use dirs::data_dir;
use once_cell::sync::Lazy;
use serde::Deserialize;
use sled::{Db, IVec};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
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
    cache: Arc<Db>,
    config: CacheConfig,
}

#[derive(Encode, Decode, Debug)]
struct HtmlCached {
    html: String,
    timestamp: i64,
}

impl Cache {
    pub fn new() -> Self {
        let cache = sled::open(get_cache_path()).expect("Failed to open cache");
        let config = Config::get_cache_config();
        Cache {
            cache: Arc::new(cache),
            config,
        }
    }

    pub fn insert(&self, key: &str, value: &str) -> Result<(), IsError> {
        match self.config.cache_mode {
            CacheMode::Read | CacheMode::Disabled => return Ok(()),
            _ => {}
        }
        let timestamp = Self::current_time()? + self.config.ttl;
        let html_cached = HtmlCached {
            html: value.to_string(),
            timestamp,
        };
        if self.cache.len() >= self.config.max_size {
            self.purge_cache()?;
        }
        log::trace!("Inserting {:?}", key.as_bytes());
        self.cache.insert(
            key.as_bytes(),
            bincode::encode_to_vec(&html_cached, standard())?,
        )?;
        self.cache.flush()?;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), IsError> {
        let result = self.cache.iter().try_for_each(|result| {
            let (url, _) = result?;
            log::trace!("Removed from command {:?}", &url);
            self.cache.remove(url)?;
            Ok(())
        });
        self.cache.flush()?;
        result
    }

    fn current_time() -> Result<i64, IsError> {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64)
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, IsError> {
        match self.config.cache_mode {
            CacheMode::Write | CacheMode::Disabled => return Ok(None),
            _ => {}
        }
        let enc_key = key.as_bytes();
        log::trace!("Fetch {:?}", enc_key);
        if let Some(data) = self.cache.get(enc_key)? {
            if let Ok((entry, _)) =
                bincode::borrow_decode_from_slice::<HtmlCached, _>(&data, standard())
            {
                if entry.timestamp < Self::current_time()? {
                    self.cache.remove(enc_key)?;
                    log::trace!("Removed expired {:?}", enc_key);

                    return Ok(None);
                }
                return Ok(Some(entry.html));
            }
        }
        Ok(None)
    }

    fn purge_cache(&self) -> Result<(), IsError> {
        let current_time = Self::current_time()?;
        let mut sorted: Vec<(IVec, i64)> = self
            .cache
            .iter()
            .filter_map(|item| {
                item.map_err(|e| IsError::Cache(e.to_string()))
                    .and_then(|(key, value)| self.remove_expired(current_time, key, value))
                    .unwrap_or_else(|e| {
                        log::error!("Failed to purge expired {:?}", e);
                        None
                    })
            })
            .collect();
        sorted.sort_by_key(|(_, timestamp)| *timestamp);
        let elements_to_retain = ((self.config.max_size * (100 - self.config.cull as usize) + 99)
            / 100)
            .min(sorted.len());

        let remove = sorted.len().saturating_sub(elements_to_retain);

        sorted.iter().take(remove).try_for_each(|(url, _)| {
            self.cache.remove(url)?;
            log::trace!("Removed due to max size hit {:?}", url);
            Ok::<(), IsError>(())
        })
    }

    fn remove_expired(
        &self,
        current_time: i64,
        key: IVec,
        value: IVec,
    ) -> Result<Option<(IVec, i64)>, IsError> {
        let (html, _) = bincode::borrow_decode_from_slice::<HtmlCached, _>(&value, standard())?;
        let expires = html.timestamp;
        if expires < current_time {
            log::trace!("Remove expired {:?}", key.to_vec());
            self.cache.remove(key)?;
            return Ok(None);
        }
        Ok(Some((key, expires)))
    }
}

pub fn cached_pages_write(url: String, html: String) {
    HTML_CACHE
        .insert(&url, &html)
        .unwrap_or_else(|e| log::error!("Error when writing page to cache: {:?}", e));
}

pub fn cached_pages_read(url: String) -> Option<String> {
    HTML_CACHE.get(&url).unwrap_or_else(|e| {
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
    path.push("is-fast-cache");
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use std::thread::sleep;
    use std::time::Duration;
    use tempfile::TempDir;

    const MS_IN_SECOND: i64 = 1000;

    #[test]
    #[serial]
    fn test_cache_can_be_read_and_written_to() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        env::set_var("XDG_DATA_HOME", path);

        let path = get_cache_path();
        let cache = sled::open(path).expect("Failed to open cache");
        let cache = Cache {
            cache: Arc::new(cache),
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

        let path = get_cache_path();
        let cache = sled::open(path).expect("Failed to open cache");
        let cache = Cache {
            cache: Arc::new(cache),
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
        sleep(Duration::from_millis(1));
        assert!(cache.get("http://localhost:8080/1").unwrap().is_none());
        assert!(cache.get("http://localhost:8080/2").unwrap().is_none());
        assert!(cache.get("http://localhost:8080/3").unwrap().is_none());
        assert!(cache.get("http://localhost:8080/4").unwrap().is_none());
        assert!(cache.get("http://localhost:8080/5").unwrap().is_none());
    }
}
