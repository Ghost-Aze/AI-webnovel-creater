use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize, PartialEq, Eq)]
#[serde(tag = "code", content = "details")]
pub enum AppError {
    #[error("{message}")]
    Validation { message: String },
    #[error("Project was not found.")]
    NotFound,
    #[error("The project is archived and cannot be updated.")]
    ArchivedProject,
    #[error("Stored project status is invalid.")]
    InvalidStatus,
    #[error("Database operation failed.")]
    Storage,
    #[error("Unexpected application error.")]
    Internal,
}

pub type AppResult<T> = Result<T, AppError>;

impl From<rusqlite::Error> for AppError {
    fn from(_: rusqlite::Error) -> Self {
        Self::Storage
    }
}

impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::Internal
    }
}
