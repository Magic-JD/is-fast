use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("Display errors: {0}")]
    DisplayError(String),

    #[error("I/O errors: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Database errors: {0}")]
    DatabaseError(#[from] rusqlite::Error),
}
