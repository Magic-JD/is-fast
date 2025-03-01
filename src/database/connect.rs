use crate::errors::error::MyError;
use crate::errors::error::MyError::DatabaseError;
use crate::links::link::Link;
use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::sync::Mutex;

static CONNECTION: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = Connection::open("is-fast.db").expect("Failed to open database");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (title TEXT, url TEXT, time DATE)",
        [],
    ).expect("Failed to create table");
    Mutex::new(conn)
});

pub fn add_history(link: &Link) -> Result<(), MyError> {
    let conn = CONNECTION.lock().unwrap();
    conn.execute(
        "INSERT INTO history (title, url, time) VALUES (?, ?, datetime('now'))",
        &[&link.title, &link.url],
    )
        .map_err(|e| DatabaseError(e))?;
    Ok(())
}

pub fn show_history() -> Result<String, MyError> {
    let conn = CONNECTION.lock().unwrap();
    let mut stmt = conn.prepare("SELECT title, url, time FROM history ORDER BY time DESC")?;
    let history = stmt.query_map([], |row| {
        Ok(
            History {
                title: row.get(0)?,
                url: row.get(1)?,
                time: row.get(2)?,
            }
        )
    }).map_err(|e| DatabaseError(e))?;
    let mut result = String::new();

    for history_item_result in history {
        let history_item = history_item_result?;
        result.push_str(&format!("{} ({})\n", history_item.title, history_item.url));
    }
    Ok(result)
}

struct History {
    title: String,
    url: String,
    time: String,
}