use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Archived,
}

impl ProjectStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }
}

impl TryFrom<&str> for ProjectStatus {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(AppError::InvalidStatus),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: ProjectStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateProjectInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateProjectInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct ProjectListFilter {
    #[serde(default)]
    pub include_archived: bool,
}

pub fn normalize_name(name: &str) -> AppResult<String> {
    let normalized = name.trim();
    if normalized.is_empty() {
        return Err(AppError::Validation {
            message: "Project name cannot be empty.".to_string(),
        });
    }
    Ok(normalized.to_string())
}

pub fn normalize_description(description: Option<&str>) -> String {
    description.unwrap_or_default().trim().to_string()
}

pub fn now_utc() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true)
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn is_utc_timestamp(value: &str) -> bool {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.offset().local_minus_utc() == 0)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_whitespace_only_names() {
        let error = normalize_name(" \t\n ").expect_err("blank names must fail");
        assert_eq!(
            error,
            AppError::Validation {
                message: "Project name cannot be empty.".to_string()
            }
        );
    }

    #[test]
    fn trims_names_and_defaults_descriptions() {
        assert_eq!(normalize_name("  A new world  ").unwrap(), "A new world");
        assert_eq!(normalize_description(None), "");
        assert_eq!(normalize_description(Some("  premise  ")), "premise");
    }

    #[test]
    fn serializes_status_values_as_lowercase_strings() {
        assert_eq!(
            serde_json::to_string(&ProjectStatus::Active).unwrap(),
            "\"active\""
        );
        assert_eq!(
            serde_json::to_string(&ProjectStatus::Archived).unwrap(),
            "\"archived\""
        );
    }

    #[test]
    fn generated_timestamps_are_utc_iso_8601() {
        assert!(is_utc_timestamp(&now_utc()));
    }
}
