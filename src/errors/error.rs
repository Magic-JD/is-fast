use thiserror::Error;

#[derive(Debug, Error)]
pub enum IsError {
    #[error("General errors: {0}")]
    General(String),

    #[error("I/O errors: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database errors: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Parse errors: {0}")]
    Parse(#[from] chrono::format::ParseError),

    #[error("Access errors: {0}")]
    Access(String),
}
