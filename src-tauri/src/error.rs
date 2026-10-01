use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize, PartialEq, Eq)]
#[serde(tag = "code", content = "details")]
pub enum AppError {
    #[error("{message}")]
    Validation { message: String },
    #[serde(rename = "not_found")]
    #[error("Project was not found.")]
    NotFound,
    #[serde(rename = "archived_project")]
    #[error("The project is archived and cannot be updated.")]
    ArchivedProject,
    #[serde(rename = "invalid_status")]
    #[error("Stored project status is invalid.")]
    InvalidStatus,
    #[serde(rename = "storage")]
    #[error("Database operation failed.")]
    Storage,
    #[serde(rename = "internal")]
    #[error("Unexpected application error.")]
    Internal,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation { .. } => "validation",
            Self::NotFound => "not_found",
            Self::ArchivedProject => "archived_project",
            Self::InvalidStatus => "invalid_status",
            Self::Storage => "storage",
            Self::Internal => "internal",
        }
    }
}

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
