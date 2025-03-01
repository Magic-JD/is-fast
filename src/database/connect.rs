use crate::errors::error::MyError;
use crate::errors::error::MyError::DatabaseError;
use crate::links::link::Link;
use dirs::data_dir;
use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

static CONNECTION: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = Connection::open(get_database_path()).expect("Failed to open database");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (title TEXT, url TEXT, time DATETIME)",
        [],
    )
    .expect("Failed to create table");
    Mutex::new(conn)
});

fn get_database_path() -> PathBuf {
    let mut path = data_dir().expect("Failed to determine data directory");
    path.push("is-fast");
    fs::create_dir_all(&path).expect("Failed to create database directory");
    path.push("is-fast.db");
    path
}

pub fn add_history(link: &Link) -> Result<(), MyError> {
    let url = remove_http(link);
    let conn = CONNECTION.lock().unwrap();
    if url_exists(&url, &conn) {
        update_row(&url, &conn)?
    } else {
        insert_row(&link, &url, conn)?
    }
}

fn insert_row(link: &&Link, url: &String, conn: MutexGuard<Connection>) -> Result<Result<(), MyError>, MyError> {
    conn.execute(
        "INSERT INTO history (title, url, time) VALUES (?, ?, datetime('now'))",
        &[&link.title, &url],
    )
        .map_err(|e| DatabaseError(e))?;
    Ok(Ok(()))
}

fn update_row(url: &String, conn: &MutexGuard<Connection>) -> Result<Result<(), MyError>, MyError> {
    conn.execute(
        "UPDATE history SET time = datetime('now') WHERE url = ?",
        &[&url],
    )
        .map_err(|e| DatabaseError(e))?;
    Ok(Ok(()))
}

fn url_exists(url: &String, conn: &MutexGuard<Connection>) -> bool {
    conn
        .query_row(
            "SELECT 1 FROM history WHERE url = ? LIMIT 1",
            &[&url],
            |row| row.get::<_, i32>(0),
        )
        .map(|_| true)
        .unwrap_or(false)
}

fn remove_http(link: &Link) -> String {
    let mut url = link.url.clone();
    if url.starts_with("https") {
        url = url[8..].to_string();
    }
    if url.starts_with("http") {
        url = url[7..].to_string();
    }
    url
}

pub fn get_history_item(index: usize) -> Result<HistoryData, MyError> {
    let history = get_history()?;
    let adjusted_index = index.saturating_sub(1);
    history
        .get(adjusted_index)
        .map(|item| item.clone())
        .ok_or(MyError::DisplayError(format!(
            "Item {} does not exist",
            index
        )))
}

pub fn get_history() -> Result<Vec<HistoryData>, MyError> {
    let conn = CONNECTION.lock().unwrap();
    let mut index = 0;
    let mut stmt = conn.prepare("SELECT title, url, time FROM history ORDER BY time DESC")?;
    let history: Vec<HistoryData> = stmt
        .query_map([], |row| {
            index += 1;
            Ok(HistoryData {
                title: row.get(0)?,
                url: row.get(1)?,
                time: row.get(2)?,
                index,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(history)
}

#[derive(Clone)]
pub struct HistoryData {
    pub(crate) title: String,
    pub(crate) url: String,
    pub(crate) time: String,
    pub(crate) index: usize,
}
