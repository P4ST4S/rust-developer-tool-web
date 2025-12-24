use std::fmt;

#[derive(Debug)]
pub enum AppError {
    ProcessFailed(std::io::Error),
    LogParsingError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ProcessFailed(e) => write!(f, "Process failed: {}", e),
            AppError::LogParsingError(s) => write!(f, "Log parsing error: {}", s),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::ProcessFailed(err)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
