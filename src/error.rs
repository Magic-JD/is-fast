use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("Custom error: {0}")]
    DisplayError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}