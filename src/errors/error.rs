use thiserror::Error;

#[derive(Debug, Error)]
pub enum IsError {
    #[error("Display errors: {0}")]
    Display(String),

    #[error("I/O errors: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database errors: {0}")]
    Database(#[from] rusqlite::Error),
}
