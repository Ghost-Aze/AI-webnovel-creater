use serde::Serialize;
use thiserror::Error;

use crate::provider::ProviderError;

#[derive(Debug, Error, Serialize, PartialEq, Eq)]
#[serde(tag = "code", content = "details")]
pub enum AppError {
    #[serde(rename = "validation")]
    #[error("{message}")]
    Validation { message: String },
    #[serde(rename = "not_found")]
    #[error("Project was not found.")]
    NotFound,
    #[serde(rename = "archived_project")]
    #[error("The project is archived and cannot be updated.")]
    ArchivedProject,
    #[serde(rename = "archived_character")]
    #[error("The character is archived and cannot be updated.")]
    ArchivedCharacter,
    #[serde(rename = "duplicate_name")]
    #[error("A character with this name already exists in the project.")]
    DuplicateName,
    #[serde(rename = "invalid_status")]
    #[error("Stored project status is invalid.")]
    InvalidStatus,
    #[serde(rename = "conflict")]
    #[error("The record changed since it was loaded.")]
    Conflict,
    #[serde(rename = "locked_canon")]
    #[error("Locked canon cannot be changed by this operation.")]
    LockedCanon,
    #[serde(rename = "invalid_proposal")]
    #[error("The proposal is invalid for its current lifecycle state.")]
    InvalidProposal,
    #[serde(rename = "provider_not_found")]
    #[error("The requested provider is not available.")]
    ProviderNotFound,
    #[serde(rename = "model_not_found")]
    #[error("The requested model is not available.")]
    ModelNotFound,
    #[serde(rename = "unsupported_capability")]
    #[error("The provider does not support this capability.")]
    UnsupportedCapability,
    #[serde(rename = "invalid_provider_request")]
    #[error("The provider request is invalid.")]
    InvalidProviderRequest,
    #[serde(rename = "provider_failure")]
    #[error("The provider operation failed.")]
    ProviderFailure,
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
            Self::ArchivedCharacter => "archived_character",
            Self::DuplicateName => "duplicate_name",
            Self::InvalidStatus => "invalid_status",
            Self::Conflict => "conflict",
            Self::LockedCanon => "locked_canon",
            Self::InvalidProposal => "invalid_proposal",
            Self::ProviderNotFound => "provider_not_found",
            Self::ModelNotFound => "model_not_found",
            Self::UnsupportedCapability => "unsupported_capability",
            Self::InvalidProviderRequest => "invalid_provider_request",
            Self::ProviderFailure => "provider_failure",
            Self::Storage => "storage",
            Self::Internal => "internal",
        }
    }
}

impl From<ProviderError> for AppError {
    fn from(error: ProviderError) -> Self {
        match error {
            ProviderError::ProviderNotFound => Self::ProviderNotFound,
            ProviderError::ModelNotFound => Self::ModelNotFound,
            ProviderError::UnsupportedCapability { .. } => Self::UnsupportedCapability,
            ProviderError::InvalidRequest => Self::InvalidProviderRequest,
            ProviderError::ProviderFailure => Self::ProviderFailure,
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
