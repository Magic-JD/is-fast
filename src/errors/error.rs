use thiserror::Error;

#[derive(Debug, Error)]
pub enum IsError {
    #[error("General errors: {0}")]
    General(String),

    #[error("Time error: {0}")]
    Time(#[from] std::time::SystemTimeError),

    #[error("I/O errors: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encode errors: {0}")]
    Encode(#[from] bincode::error::EncodeError),

    #[error("Decode errors: {0}")]
    Decode(#[from] bincode::error::DecodeError),

    #[error("Database errors: {0}")]
    DatabaseSql(#[from] rusqlite::Error),

    #[error("Parse errors: {0}")]
    Parse(#[from] chrono::format::ParseError),

    #[error("Access errors: {0}")]
    Access(String),

    #[error("Scrape errors: {0}")]
    Scrape(String),

    #[error("Search errors: {0}")]
    Search(String),

    #[error("Selector errors: {0}")]
    Selector(String),

    #[error("Csv errors: {0}")]
    Csv(String),
}
