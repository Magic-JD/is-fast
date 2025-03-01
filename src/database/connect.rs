use std::fs;
use std::path::PathBuf;
use crate::errors::error::MyError;
use crate::errors::error::MyError::DatabaseError;
use crate::links::link::Link;
use chrono::{NaiveDateTime, Utc};
use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::sync::{Mutex, MutexGuard};
use dirs::data_dir;

static CONNECTION: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = Connection::open(get_database_path()).expect("Failed to open database");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (title TEXT, url TEXT, time DATETIME)",
        [],
    ).expect("Failed to create table");
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
    let conn = CONNECTION.lock().unwrap();
    conn.execute(
        "INSERT INTO history (title, url, time) VALUES (?, ?, datetime('now'))",
        &[&link.title, &link.url],
    )
        .map_err(|e| DatabaseError(e))?;
    Ok(())
}

pub fn get_history_item(index: usize) -> Result<History, MyError> {
    let conn = CONNECTION.lock().unwrap();
    let history = get_history(conn)?;
    let adjusted_index = index.saturating_sub(1);
    history.get(adjusted_index).map(|item| item.clone()).ok_or(MyError::DisplayError(format!("Item {} does not exist", index)))

}

pub fn show_history() -> Result<String, MyError> {
    let conn = CONNECTION.lock().unwrap();
    let history = get_history(conn)?;
    let mut result = String::new();

    let mut count = 1;
    for history_item in history {
        result.insert_str(0, &format!("{} | {} | {} ({})\n", count, date_to_display(history_item.time), clip_if_needed(history_item.title, 100), clip_if_needed(history_item.url, 30)));
        count += 1;
    }
    Ok(result)
}

fn get_history(conn: MutexGuard<Connection>) -> Result<Vec<History>, MyError> {
    let mut stmt = conn.prepare("SELECT title, url, time FROM history ORDER BY time DESC LIMIT 20")?;
    let history: Vec<History> = stmt
        .query_map([], |row| {
            Ok(History {
                title: row.get(0)?,
                url: row.get(1)?,
                time: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(history)
}

fn clip_if_needed(text: String, max_length: usize) -> String {
    if text.len() > max_length {
        return format!("{}...", &text[0..max_length-3]);
    }
    text.to_string()
}

fn date_to_display(date: String) -> String {
    let now = Utc::now();
    NaiveDateTime::parse_from_str(&*date, "%Y-%m-%d %H:%M:%S")
        .map(|parsed_datetime| {parsed_datetime.and_utc()})
        .map(|datetime_utc| now.signed_duration_since(datetime_utc))
        .map(|duration| {
            if duration.num_weeks() > 0 {
                return format!("{} weeks", duration.num_weeks());
            }
            if duration.num_days() > 0 {
                return format!("{} days", duration.num_days());

            }
            if duration.num_hours() > 0 {
                return format!("{} hours", duration.num_hours());
            }
            if duration.num_minutes() > 0 {
                return format!("{} minutes", duration.num_minutes());
            }
            if duration.num_seconds() > 0 {
                return format!("{} seconds", duration.num_seconds());
            }
            "Date could not be displayed".to_string()
        })
        .unwrap_or_else(|_| "Date could not be displayed".to_string())
}

#[derive(Clone)]
pub struct History {
    pub(crate) title: String,
    pub(crate) url: String,
    time: String,
}