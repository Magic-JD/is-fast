use crate::config::files::database_path;
use crate::errors::error::IsError;
use crate::errors::error::IsError::{Access, DatabaseSql};
use chrono::NaiveDateTime;
use once_cell::sync::Lazy;
use rusqlite::types::Type;
use rusqlite::Error::FromSqlConversionFailure;
use rusqlite::{Connection, Error, Row};
use std::sync::{Mutex, MutexGuard};

static CONNECTION: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = Connection::open(database_path()).expect("Failed to open database");
    conn.execute_batch(
        "BEGIN TRANSACTION;
     CREATE TABLE IF NOT EXISTS history (
         title TEXT,
         url TEXT,
         time DATETIME
     );
     CREATE INDEX IF NOT EXISTS idx_url ON history (url);
     COMMIT;",
    )
    .expect("Failed to initialize database");
    Mutex::new(conn)
});

pub fn add_history(title: &str, link: &str) -> Result<(), IsError> {
    let url = remove_http(link);
    let conn = CONNECTION.lock().map_err(|e| Access(e.to_string()))?;
    if url_exists(&url, &conn) {
        update_row(title, &url, &conn)
    } else {
        insert_row(title, &url, &conn)
    }
}

fn insert_row(title: &str, url: &str, conn: &MutexGuard<Connection>) -> Result<(), IsError> {
    conn.execute(
        "INSERT INTO history (title, url, time) VALUES (?, ?, datetime('now'))",
        [title, url],
    )
    .map_err(DatabaseSql)?;
    Ok(())
}

fn update_row(title: &str, url: &str, conn: &MutexGuard<Connection>) -> Result<(), IsError> {
    conn.execute(
        "UPDATE history SET time = datetime('now'), title = ? WHERE url = ?",
        [title, url],
    )
    .map_err(DatabaseSql)?;
    Ok(())
}

fn url_exists(url: &str, conn: &MutexGuard<Connection>) -> bool {
    conn.query_row(
        "SELECT 1 FROM history WHERE url = ? LIMIT 1",
        [url],
        |row| row.get::<_, i32>(0),
    )
    .map(|_| true)
    .unwrap_or(false)
}

pub fn get_history() -> Result<Vec<HistoryData>, IsError> {
    let conn = CONNECTION.lock().map_err(|e| Access(e.to_string()))?;
    let mut stmt = conn.prepare("SELECT title, url, time FROM history ORDER BY time DESC")?;
    let history: Vec<HistoryData> = stmt
        .query_map([], convert_to_history_data)?
        .collect::<Result<_, _>>()
        .map_err(DatabaseSql)?;
    log::debug!("Retrieved {} history items", history.len());
    Ok(history)
}

pub fn get_latest_history() -> Result<Option<HistoryData>, IsError> {
    let conn = CONNECTION.lock().map_err(|e| Access(e.to_string()))?;
    let mut stmt =
        conn.prepare("SELECT title, url, time FROM history ORDER BY time DESC LIMIT 1")?;

    let mut rows = stmt.query_map([], convert_to_history_data)?;

    if let Some(result) = rows.next().transpose().map_err(DatabaseSql)? {
        log::debug!("Retrieved latest history item - {}", result.title);
        Ok(Some(result))
    } else {
        log::debug!("No latest history item found");
        Ok(None) // No history available
    }
}

pub fn clear_history() -> Result<(), IsError> {
    let conn = CONNECTION.lock().map_err(|e| Access(e.to_string()))?;
    conn.execute("DROP TABLE history", [])?;
    Ok(())
}

pub fn remove_history(url: &str) -> Result<(), IsError> {
    let conn = CONNECTION.lock().map_err(|e| Access(e.to_string()))?;
    conn.execute("DELETE FROM history WHERE url = ?", [url])
        .map_err(DatabaseSql)?;
    Ok(())
}

fn remove_http(url: &str) -> String {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url)
        .to_string()
}

#[derive(Clone, PartialEq)]
pub struct HistoryData {
    pub(crate) title: String,
    pub(crate) url: String,
    pub(crate) time: NaiveDateTime,
}

fn convert_to_history_data(row: &Row) -> Result<HistoryData, Error> {
    let time_str: String = row.get(2)?;
    let time = NaiveDateTime::parse_from_str(&time_str, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| FromSqlConversionFailure(2, Type::Text, Box::new(e)))?;
    Ok(HistoryData {
        title: row.get(0)?,
        url: row.get(1)?,
        time,
    })
}
