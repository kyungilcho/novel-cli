use std::{fmt, io};

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidId(u64),
    Diesel(diesel::result::Error),
    DieselConnection(diesel::ConnectionError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Json(e) => write!(f, "JSON error: {}", e),
            AppError::InvalidId(id) => write!(f, "Invalid ID: {}", id),
            AppError::Diesel(e) => write!(f, "Diesel error: {}", e),
            AppError::DieselConnection(e) => write!(f, "Diesel connection error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        AppError::Io(value)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::Json(value)
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(value: diesel::result::Error) -> Self {
        AppError::Diesel(value)
    }
}

impl From<diesel::ConnectionError> for AppError {
    fn from(value: diesel::ConnectionError) -> Self {
        AppError::DieselConnection(value)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
